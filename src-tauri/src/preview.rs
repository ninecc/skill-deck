use crate::cli::{CliManager, CommandError};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
};

const MAX_TEXT_BYTES: u64 = 1024 * 1024;
const MAX_IMAGE_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerKind {
    Markdown,
    Text,
    Code,
    Image,
    Unsupported,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub level: usize,
    pub directory: bool,
    pub size: u64,
    pub viewer: ViewerKind,
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContent {
    pub path: String,
    pub viewer: ViewerKind,
    pub size: u64,
    pub text: Option<String>,
    pub data_url: Option<String>,
    pub translatable: bool,
}

pub fn tree(manager: &CliManager, skill: &str) -> Result<Vec<FileEntry>, CommandError> {
    let root = canonical_root(manager, skill)?;
    let mut entries = Vec::new();
    walk(&root, &root, 1, &mut entries)?;
    Ok(entries)
}

pub fn read(
    manager: &CliManager,
    skill: &str,
    relative_path: &str,
) -> Result<FileContent, CommandError> {
    let root = canonical_root(manager, skill)?;
    let path = contained_file(&root, relative_path)?;
    let metadata =
        fs::symlink_metadata(&path).map_err(|error| io_error("inspect", &path, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CommandError::new(
            "unsupported_file",
            "Only regular files inside the installed Skill can be previewed.",
        ));
    }
    let size = metadata.len();
    let viewer = classify(&path);
    let bytes = match viewer {
        ViewerKind::Image if size <= MAX_IMAGE_BYTES => read_at_most(&path, MAX_IMAGE_BYTES),
        ViewerKind::Markdown | ViewerKind::Text | ViewerKind::Code if size <= MAX_TEXT_BYTES => {
            read_at_most(&path, MAX_TEXT_BYTES)
        }
        ViewerKind::Image => {
            return Err(CommandError::new(
                "file_too_large",
                "The image exceeds the 10 MiB preview limit.",
            ))
        }
        ViewerKind::Markdown | ViewerKind::Text | ViewerKind::Code => {
            return Err(CommandError::new(
                "file_too_large",
                "The file exceeds the 1 MiB text preview limit.",
            ));
        }
        ViewerKind::Unsupported => {
            return Err(CommandError::new(
                "unsupported_file",
                "This file type cannot be previewed inside Skill Deck.",
            ))
        }
    }?;
    let (text, data_url) = if viewer == ViewerKind::Image {
        let mime = image_mime(&path);
        (
            None,
            Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes))),
        )
    } else {
        let text = String::from_utf8(bytes).map_err(|_| {
            CommandError::new(
                "invalid_encoding",
                "The selected text file is not valid UTF-8.",
            )
        })?;
        (Some(text), None)
    };
    Ok(FileContent {
        path: relative_path.to_owned(),
        viewer,
        size,
        text,
        data_url,
        translatable: matches!(viewer, ViewerKind::Markdown | ViewerKind::Text),
    })
}

pub fn reveal(
    manager: &CliManager,
    skill: &str,
    relative_path: Option<&str>,
) -> Result<(), CommandError> {
    let root = canonical_root(manager, skill)?;
    let target = match relative_path {
        Some(path) => contained_entry(&root, path)?,
        None => root,
    };
    #[cfg(target_os = "macos")]
    let status = if relative_path.is_some() {
        Command::new("open").arg("-R").arg(&target).status()
    } else {
        Command::new("open").arg(&target).status()
    };
    #[cfg(target_os = "windows")]
    let status = if relative_path.is_some() {
        Command::new("explorer")
            .arg(format!("/select,{}", target.display()))
            .status()
    } else {
        Command::new("explorer").arg(&target).status()
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open")
        .arg(if target.is_dir() {
            &target
        } else {
            target.parent().unwrap_or(&target)
        })
        .status();
    status
        .map_err(|error| {
            CommandError::new(
                "reveal_failed",
                format!("Could not open the system file manager: {error}"),
            )
        })?
        .success()
        .then_some(())
        .ok_or_else(|| {
            CommandError::new(
                "reveal_failed",
                "The system file manager could not reveal the selected path.",
            )
        })
}

fn canonical_root(manager: &CliManager, skill: &str) -> Result<PathBuf, CommandError> {
    manager.skill_root(skill)?.canonicalize().map_err(|error| {
        CommandError::new(
            "skill_unavailable",
            format!("Could not resolve the installed Skill root: {error}"),
        )
    })
}

fn contained_file(root: &Path, relative_path: &str) -> Result<PathBuf, CommandError> {
    let relative = validated_relative_path(relative_path)?;
    let joined = root.join(relative);
    let resolved = joined
        .canonicalize()
        .map_err(|error| io_error("resolve", &joined, error))?;
    if !resolved.starts_with(root) {
        return Err(CommandError::new(
            "path_outside_skill",
            "The selected file resolves outside the installed Skill.",
        ));
    }
    Ok(joined)
}

fn contained_entry(root: &Path, relative_path: &str) -> Result<PathBuf, CommandError> {
    let joined = root.join(validated_relative_path(relative_path)?);
    let parent = joined
        .parent()
        .and_then(|path| path.canonicalize().ok())
        .ok_or_else(|| CommandError::new("io", "Could not resolve the selected file's parent."))?;
    if !parent.starts_with(root) {
        return Err(CommandError::new(
            "path_outside_skill",
            "The selected file is outside the installed Skill.",
        ));
    }
    fs::symlink_metadata(&joined).map_err(|error| io_error("inspect", &joined, error))?;
    Ok(joined)
}

fn validated_relative_path(relative_path: &str) -> Result<&Path, CommandError> {
    let relative = Path::new(relative_path);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CommandError::new(
            "invalid_path",
            "Preview paths must be relative paths inside the selected Skill.",
        ));
    }
    Ok(relative)
}

fn walk(
    root: &Path,
    directory: &Path,
    level: usize,
    entries: &mut Vec<FileEntry>,
) -> Result<(), CommandError> {
    let mut children: Vec<_> = fs::read_dir(directory)
        .map_err(|error| io_error("list", directory, error))?
        .collect::<Result<_, _>>()
        .map_err(|error| io_error("list", directory, error))?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    for child in children {
        let path = child.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| io_error("inspect", &path, error))?;
        let relative = path
            .strip_prefix(root)
            .expect("walked path remains below root");
        let logical_path = relative.to_string_lossy().replace('\\', "/");
        let directory = metadata.is_dir();
        let symlink = metadata.file_type().is_symlink();
        let regular = metadata.is_file();
        let viewer = if directory || symlink || !regular {
            ViewerKind::Unsupported
        } else {
            classify(&path)
        };
        let limit = if viewer == ViewerKind::Image {
            MAX_IMAGE_BYTES
        } else {
            MAX_TEXT_BYTES
        };
        let unsupported_reason = if symlink {
            Some("Links are listed but are not followed or previewed.".into())
        } else if !directory && !regular {
            Some("Special files cannot be previewed.".into())
        } else if !directory && viewer == ViewerKind::Unsupported {
            Some("This file type has no inline viewer.".into())
        } else if !directory && metadata.len() > limit {
            Some(format!(
                "File exceeds the {} MiB preview limit.",
                limit / 1024 / 1024
            ))
        } else {
            None
        };
        entries.push(FileEntry {
            path: if directory {
                format!("{logical_path}/")
            } else {
                logical_path
            },
            name: child.file_name().to_string_lossy().into_owned(),
            level,
            directory,
            size: metadata.len(),
            viewer,
            unsupported_reason,
        });
        if directory {
            walk(root, &path, level + 1, entries)?;
        }
    }
    Ok(())
}

fn read_at_most(path: &Path, limit: u64) -> Result<Vec<u8>, CommandError> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(limit + 1).read_to_end(&mut bytes))
        .map_err(|error| io_error("read", path, error))?;
    if bytes.len() as u64 > limit {
        return Err(CommandError::new(
            "file_too_large",
            "The file grew beyond its preview limit while it was being read.",
        ));
    }
    Ok(bytes)
}

fn classify(path: &Path) -> ViewerKind {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "md" | "markdown" | "mdx" => ViewerKind::Markdown,
        "txt" | "rst" | "adoc" => ViewerKind::Text,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => ViewerKind::Image,
        "json" | "jsonl" | "yaml" | "yml" | "toml" | "xml" | "html" | "css" | "js" | "jsx"
        | "ts" | "tsx" | "py" | "rb" | "rs" | "go" | "java" | "kt" | "swift" | "sh" | "bash"
        | "zsh" | "fish" | "sql" | "csv" => ViewerKind::Code,
        _ => ViewerKind::Unsupported,
    }
}

fn image_mime(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}

fn io_error(operation: &str, path: &Path, error: std::io::Error) -> CommandError {
    CommandError::new(
        "io",
        format!("Could not {operation} {}: {error}", path.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_reject_traversal_and_absolute_inputs() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            contained_file(root.path(), "../secret").unwrap_err().code,
            "invalid_path"
        );
        assert_eq!(
            contained_file(root.path(), "/tmp/secret").unwrap_err().code,
            "invalid_path"
        );
    }

    #[cfg(unix)]
    #[test]
    fn reveal_containment_accepts_a_link_entry_without_following_it() {
        let root = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("/outside", root.path().join("linked")).unwrap();
        let canonical = root.path().canonicalize().unwrap();
        assert_eq!(
            contained_entry(&canonical, "linked").unwrap(),
            canonical.join("linked")
        );
        assert_eq!(contained_file(&canonical, "linked").unwrap_err().code, "io");
    }

    #[test]
    fn viewer_classification_limits_translation_to_docs() {
        assert_eq!(classify(Path::new("SKILL.md")), ViewerKind::Markdown);
        assert_eq!(classify(Path::new("config.yaml")), ViewerKind::Code);
        assert_eq!(classify(Path::new("archive.zip")), ViewerKind::Unsupported);
    }

    #[test]
    fn bounded_reads_reject_growth_past_the_limit() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("file.txt");
        fs::write(&path, b"four").unwrap();
        assert_eq!(read_at_most(&path, 3).unwrap_err().code, "file_too_large");
        assert_eq!(read_at_most(&path, 4).unwrap(), b"four");
    }
}
