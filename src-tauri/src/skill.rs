use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_yaml::Value;
use sha2::{Digest, Sha256};

const MIB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy)]
struct ResourcePolicy {
    package_bytes: u64,
    file_count: u64,
    single_file_bytes: u64,
    skill_markdown_bytes: u64,
}

impl Default for ResourcePolicy {
    fn default() -> Self {
        Self {
            package_bytes: 100 * MIB,
            file_count: 10_000,
            single_file_bytes: 50 * MIB,
            skill_markdown_bytes: MIB,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillErrorCode {
    AgentRootMissing,
    Busy,
    ConfigurationDrift,
    ConfigurationExternallyControlled,
    Conflict,
    ContentDrift,
    CopyFallbackRequired,
    InvalidStructure,
    InvalidMetadata,
    InvalidPlan,
    LegacyConflict,
    ResourceLimitExceeded,
    SourceMissing,
    SourceUnreachable,
    SourceChanged,
    TopologyChanged,
    UnsupportedGitFeature,
    UnsupportedFileType,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillError {
    pub code: SkillErrorCode,
    pub message: String,
    pub path: Option<PathBuf>,
    pub limit: Option<u64>,
    pub observed: Option<u64>,
}

impl SkillError {
    pub(crate) fn new(
        code: SkillErrorCode,
        message: impl Into<String>,
        path: Option<PathBuf>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path,
            limit: None,
            observed: None,
        }
    }

    pub(crate) fn limit(
        message: impl Into<String>,
        path: Option<PathBuf>,
        limit: u64,
        observed: u64,
    ) -> Self {
        Self {
            code: SkillErrorCode::ResourceLimitExceeded,
            message: message.into(),
            path,
            limit: Some(limit),
            observed: Some(observed),
        }
    }

    pub(crate) fn io(path: &Path, error: std::io::Error) -> Self {
        Self::new(
            SkillErrorCode::Io,
            format!("Could not inspect skill source: {error}"),
            Some(path.to_path_buf()),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub unknown_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceObservation {
    pub package_bytes: u64,
    pub file_count: u64,
    pub largest_file_bytes: u64,
    pub skill_markdown_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedSkill {
    pub root: PathBuf,
    pub fingerprint: String,
    pub metadata: SkillMetadata,
    pub resources: ResourceObservation,
    pub scripts: Vec<PathBuf>,
    pub references: Vec<PathBuf>,
}

pub fn validate_skill_dir(root: &Path) -> Result<ValidatedSkill, SkillError> {
    validate_with_policy(root, ResourcePolicy::default(), None)
}

pub(crate) fn validate_installed_revision(
    root: &Path,
    expected_name: &str,
) -> Result<ValidatedSkill, SkillError> {
    validate_with_policy(root, ResourcePolicy::default(), Some(expected_name))
}

fn validate_with_policy(
    root: &Path,
    policy: ResourcePolicy,
    expected_name: Option<&str>,
) -> Result<ValidatedSkill, SkillError> {
    let root_metadata = fs::symlink_metadata(root).map_err(|error| SkillError::io(root, error))?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Skill source must be a real directory, not a link or special file",
            Some(root.to_path_buf()),
        ));
    }

    let mut observation = ResourceObservation {
        package_bytes: 0,
        file_count: 0,
        largest_file_bytes: 0,
        skill_markdown_bytes: 0,
    };
    let mut scripts = Vec::new();
    let mut references = Vec::new();
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop() {
        let entries =
            fs::read_dir(&directory).map_err(|error| SkillError::io(&directory, error))?;
        for entry in entries {
            let entry = entry.map_err(|error| SkillError::io(&directory, error))?;
            let path = entry.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|error| SkillError::io(&path, error))?;
            let file_type = metadata.file_type();

            if file_type.is_symlink() {
                return Err(SkillError::new(
                    SkillErrorCode::UnsupportedFileType,
                    "Skill packages cannot contain symbolic links or junctions",
                    Some(path),
                ));
            }
            if file_type.is_dir() {
                pending.push(path);
                continue;
            }
            if !file_type.is_file() {
                return Err(SkillError::new(
                    SkillErrorCode::UnsupportedFileType,
                    "Skill packages can contain only regular files and directories",
                    Some(path),
                ));
            }

            observation.file_count += 1;
            if observation.file_count > policy.file_count {
                return Err(SkillError::limit(
                    "Skill package contains too many files",
                    Some(path),
                    policy.file_count,
                    observation.file_count,
                ));
            }

            let bytes = metadata.len();
            if bytes > policy.single_file_bytes {
                return Err(SkillError::limit(
                    "A file exceeds the single-file size limit",
                    Some(path),
                    policy.single_file_bytes,
                    bytes,
                ));
            }
            observation.package_bytes = observation.package_bytes.saturating_add(bytes);
            observation.largest_file_bytes = observation.largest_file_bytes.max(bytes);
            if observation.package_bytes > policy.package_bytes {
                return Err(SkillError::limit(
                    "Skill package exceeds the total size limit",
                    Some(root.to_path_buf()),
                    policy.package_bytes,
                    observation.package_bytes,
                ));
            }

            let relative = path
                .strip_prefix(root)
                .expect("walked path stays under root");
            files.push((relative.to_path_buf(), path.clone(), bytes));
            if relative == Path::new("SKILL.md") {
                observation.skill_markdown_bytes = bytes;
                if bytes > policy.skill_markdown_bytes {
                    return Err(SkillError::limit(
                        "SKILL.md exceeds its size limit",
                        Some(path.clone()),
                        policy.skill_markdown_bytes,
                        bytes,
                    ));
                }
            }
            if relative.starts_with("scripts") {
                scripts.push(relative.to_path_buf());
            } else if relative.starts_with("references") {
                references.push(relative.to_path_buf());
            }
        }
    }

    scripts.sort();
    references.sort();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let skill_markdown = root.join("SKILL.md");
    if observation.skill_markdown_bytes == 0 && !skill_markdown.is_file() {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Skill package must contain SKILL.md at its root",
            Some(skill_markdown),
        ));
    }

    let content = fs::read_to_string(&skill_markdown)
        .map_err(|error| SkillError::io(&skill_markdown, error))?;
    let metadata = parse_metadata(&content, &skill_markdown)?;
    let directory_name = root.file_name().and_then(|name| name.to_str());
    if expected_name.unwrap_or_else(|| directory_name.unwrap_or_default()) != metadata.name {
        return Err(SkillError::new(
            SkillErrorCode::InvalidMetadata,
            "Skill directory name must exactly match frontmatter name",
            Some(root.to_path_buf()),
        ));
    }

    Ok(ValidatedSkill {
        root: root.to_path_buf(),
        fingerprint: fingerprint(&files)?,
        metadata,
        resources: observation,
        scripts,
        references,
    })
}

fn fingerprint(files: &[(PathBuf, PathBuf, u64)]) -> Result<String, SkillError> {
    let mut digest = Sha256::new();
    for (relative, path, expected_bytes) in files {
        let relative = relative.to_string_lossy();
        digest.update((relative.len() as u64).to_le_bytes());
        digest.update(relative.as_bytes());
        digest.update(expected_bytes.to_le_bytes());

        let mut file = fs::File::open(path).map_err(|error| SkillError::io(path, error))?;
        let mut observed_bytes = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| SkillError::io(path, error))?;
            if read == 0 {
                break;
            }
            observed_bytes += read as u64;
            digest.update(&buffer[..read]);
        }
        if observed_bytes != *expected_bytes {
            return Err(SkillError::new(
                SkillErrorCode::Io,
                "Skill source changed while it was being inspected",
                Some(path.clone()),
            ));
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn parse_metadata(content: &str, path: &Path) -> Result<SkillMetadata, SkillError> {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return Err(SkillError::new(
            SkillErrorCode::InvalidMetadata,
            "SKILL.md must start with YAML frontmatter",
            Some(path.to_path_buf()),
        ));
    }

    let mut yaml = String::new();
    let mut closed = false;
    for line in lines {
        if line == "---" {
            closed = true;
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    if !closed {
        return Err(SkillError::new(
            SkillErrorCode::InvalidMetadata,
            "SKILL.md frontmatter is missing its closing delimiter",
            Some(path.to_path_buf()),
        ));
    }

    let Value::Mapping(mapping) = serde_yaml::from_str::<Value>(&yaml).map_err(|error| {
        SkillError::new(
            SkillErrorCode::InvalidMetadata,
            format!("SKILL.md frontmatter is invalid YAML: {error}"),
            Some(path.to_path_buf()),
        )
    })?
    else {
        return Err(SkillError::new(
            SkillErrorCode::InvalidMetadata,
            "SKILL.md frontmatter must be a YAML mapping",
            Some(path.to_path_buf()),
        ));
    };

    let mut fields = BTreeMap::new();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err(SkillError::new(
                SkillErrorCode::InvalidMetadata,
                "SKILL.md frontmatter keys must be strings",
                Some(path.to_path_buf()),
            ));
        };
        fields.insert(key.to_owned(), value);
    }

    let name = take_required_string(&mut fields, "name", path)?;
    let description = take_required_string(&mut fields, "description", path)?;
    if !valid_name(&name) {
        return Err(SkillError::new(
            SkillErrorCode::InvalidMetadata,
            "Skill name must be 1-64 lowercase ASCII letters, digits, or single hyphens",
            Some(path.to_path_buf()),
        ));
    }
    if description.is_empty() || description.chars().count() > 1024 {
        return Err(SkillError::new(
            SkillErrorCode::InvalidMetadata,
            "Skill description must contain 1-1024 characters",
            Some(path.to_path_buf()),
        ));
    }

    Ok(SkillMetadata {
        name,
        description,
        unknown_fields: fields,
    })
}

fn take_required_string(
    fields: &mut BTreeMap<String, Value>,
    key: &str,
    path: &Path,
) -> Result<String, SkillError> {
    fields
        .remove(key)
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| {
            SkillError::new(
                SkillErrorCode::InvalidMetadata,
                format!("SKILL.md frontmatter requires a string {key} field"),
                Some(path.to_path_buf()),
            )
        })
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn skill(name: &str, metadata_name: &str) -> (TempDir, PathBuf) {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join(name);
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join("SKILL.md"),
            format!(
                "---\nname: {metadata_name}\ndescription: Test skill\nvendor-field:\n  enabled: true\n---\nBody\n"
            ),
        )
        .unwrap();
        (temp, root)
    }

    #[test]
    fn validates_metadata_and_discloses_structure() {
        let (_temp, root) = skill("test-skill", "test-skill");
        fs::create_dir(root.join("scripts")).unwrap();
        fs::create_dir(root.join("references")).unwrap();
        fs::write(root.join("scripts/run.sh"), "#!/bin/sh\n").unwrap();
        fs::write(root.join("references/guide.md"), "Guide\n").unwrap();

        let validated = validate_skill_dir(&root).unwrap();

        assert_eq!(validated.metadata.name, "test-skill");
        assert!(validated
            .metadata
            .unknown_fields
            .contains_key("vendor-field"));
        assert_eq!(validated.scripts, [PathBuf::from("scripts/run.sh")]);
        assert_eq!(validated.references, [PathBuf::from("references/guide.md")]);
        assert_eq!(validated.resources.file_count, 3);
    }

    #[test]
    fn rejects_invalid_or_mismatched_names() {
        let (_temp, root) = skill("folder-name", "Different_Name");
        let error = validate_skill_dir(&root).unwrap_err();
        assert_eq!(error.code, SkillErrorCode::InvalidMetadata);

        let (_temp, root) = skill("folder-name", "other-name");
        let error = validate_skill_dir(&root).unwrap_err();
        assert_eq!(
            error.message,
            "Skill directory name must exactly match frontmatter name"
        );
    }

    #[test]
    fn reports_exact_resource_limit_and_observation() {
        let (_temp, root) = skill("test-skill", "test-skill");
        let markdown_bytes = fs::metadata(root.join("SKILL.md")).unwrap().len();
        let policy = ResourcePolicy {
            package_bytes: markdown_bytes,
            file_count: 1,
            single_file_bytes: markdown_bytes,
            skill_markdown_bytes: markdown_bytes,
        };
        validate_with_policy(&root, policy, None).unwrap();

        fs::write(root.join("extra.txt"), "x").unwrap();
        let error = validate_with_policy(&root, policy, None).unwrap_err();
        assert_eq!(error.code, SkillErrorCode::ResourceLimitExceeded);
        assert_eq!(error.limit, Some(1));
        assert_eq!(error.observed, Some(2));

        let (_temp, root) = skill("test-skill", "test-skill");
        let markdown_bytes = fs::metadata(root.join("SKILL.md")).unwrap().len();
        let policy = ResourcePolicy {
            package_bytes: markdown_bytes,
            file_count: 10,
            single_file_bytes: u64::MAX,
            skill_markdown_bytes: u64::MAX,
        };
        validate_with_policy(&root, policy, None).unwrap();
        fs::write(root.join("extra.txt"), "x").unwrap();
        let error = validate_with_policy(&root, policy, None).unwrap_err();
        assert_eq!(
            (error.limit, error.observed),
            (Some(markdown_bytes), Some(markdown_bytes + 1))
        );

        let (_temp, root) = skill("test-skill", "test-skill");
        let markdown_bytes = fs::metadata(root.join("SKILL.md")).unwrap().len();
        fs::write(root.join("extra.txt"), vec![b'x'; markdown_bytes as usize]).unwrap();
        let policy = ResourcePolicy {
            package_bytes: u64::MAX,
            file_count: 10,
            single_file_bytes: markdown_bytes,
            skill_markdown_bytes: u64::MAX,
        };
        validate_with_policy(&root, policy, None).unwrap();
        fs::write(
            root.join("extra.txt"),
            vec![b'x'; markdown_bytes as usize + 1],
        )
        .unwrap();
        let error = validate_with_policy(&root, policy, None).unwrap_err();
        assert_eq!(
            (error.limit, error.observed),
            (Some(markdown_bytes), Some(markdown_bytes + 1))
        );

        let (_temp, root) = skill("test-skill", "test-skill");
        let markdown = fs::read(root.join("SKILL.md")).unwrap();
        let markdown_bytes = markdown.len() as u64;
        let policy = ResourcePolicy {
            package_bytes: u64::MAX,
            file_count: 10,
            single_file_bytes: u64::MAX,
            skill_markdown_bytes: markdown_bytes,
        };
        validate_with_policy(&root, policy, None).unwrap();
        fs::write(root.join("SKILL.md"), [markdown, vec![b'x']].concat()).unwrap();
        let error = validate_with_policy(&root, policy, None).unwrap_err();
        assert_eq!(
            (error.limit, error.observed),
            (Some(markdown_bytes), Some(markdown_bytes + 1))
        );
    }

    #[test]
    fn fingerprint_is_stable_and_content_sensitive() {
        let (_temp, root) = skill("test-skill", "test-skill");
        fs::write(root.join("b.txt"), "second").unwrap();
        fs::write(root.join("a.txt"), "first").unwrap();
        let first = validate_skill_dir(&root).unwrap().fingerprint;

        assert_eq!(first, validate_skill_dir(&root).unwrap().fingerprint);

        fs::write(root.join("a.txt"), "changed").unwrap();
        assert_ne!(first, validate_skill_dir(&root).unwrap().fingerprint);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_internal_links_without_following_them() {
        use std::os::unix::fs::symlink;

        let (_temp, root) = skill("test-skill", "test-skill");
        symlink("SKILL.md", root.join("linked.md")).unwrap();

        let error = validate_skill_dir(&root).unwrap_err();
        assert_eq!(error.code, SkillErrorCode::UnsupportedFileType);
    }
}
