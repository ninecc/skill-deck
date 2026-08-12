use crate::{
    cli::{CliManager, CommandError},
    preview,
};
use serde::Serialize;
use std::time::{Duration, Instant};

const GOOGLE_ENDPOINT: &str = "https://translate.googleapis.com/translate_a/single";
const CHUNK_CHARS: usize = 3_500;
const MAX_PROXY_BYTES: usize = 2_048;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(15);
const LANGUAGES: &[&str] = &[
    "en", "zh-Hans", "zh-Hant", "ja", "ko", "es", "fr", "de", "pt", "it", "ru", "ar", "hi",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResult {
    pub translated_text: String,
    pub detected_source_language: Option<String>,
}

pub fn translate_installed(
    manager: &CliManager,
    skill: &str,
    path: &str,
    target_language: &str,
    translation_proxy: &str,
) -> Result<TranslationResult, CommandError> {
    if !LANGUAGES.contains(&target_language) {
        return Err(CommandError::new(
            "unsupported_language",
            "The selected translation language is not supported.",
        ));
    }
    let deadline = Instant::now() + OPERATION_TIMEOUT;
    let client = translation_client(translation_proxy)?;
    let content = preview::read(manager, skill, path)?;
    if !content.translatable {
        return Err(CommandError::new(
            "not_translatable",
            "Only Markdown and plain-text documentation can be translated.",
        ));
    }
    let text = content.text.unwrap_or_default();
    let provider_target = match target_language {
        "zh-Hans" => "zh-CN",
        "zh-Hant" => "zh-TW",
        value => value,
    };
    let markdown = matches!(content.viewer, preview::ViewerKind::Markdown);
    // ponytail: anonymous endpoint is best-effort; replace this function with a supported provider when quotas or reliability matter.
    let mut detected = None;
    let translated_text = if markdown {
        map_markdown_prose(&text, |prose| {
            let result = google_translate(&client, deadline, prose, provider_target)?;
            if detected.is_none() {
                detected = result.detected_source_language;
            }
            Ok(result.translated_text)
        })?
    } else {
        let result = google_translate(&client, deadline, &text, provider_target)?;
        detected = result.detected_source_language;
        result.translated_text
    };
    Ok(TranslationResult {
        translated_text,
        detected_source_language: detected,
    })
}

fn translation_client(proxy: &str) -> Result<reqwest::blocking::Client, CommandError> {
    let mut builder = reqwest::blocking::Client::builder().connect_timeout(CONNECT_TIMEOUT);
    if !proxy.is_empty() {
        validate_proxy(proxy)?;
        builder = builder.proxy(reqwest::Proxy::all(proxy).map_err(|_| invalid_proxy())?);
    }
    builder.build().map_err(|_| provider_unavailable())
}

fn validate_proxy(proxy: &str) -> Result<(), CommandError> {
    if proxy.len() > MAX_PROXY_BYTES {
        return Err(invalid_proxy());
    }
    let url = reqwest::Url::parse(proxy).map_err(|_| invalid_proxy())?;
    let authority = proxy.split_once("://").map(|(_, value)| value);
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || authority.is_none_or(|value| value.contains(['/', '?', '#']))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid_proxy());
    }
    Ok(())
}

fn invalid_proxy() -> CommandError {
    CommandError::new(
        "invalid_proxy",
        "Use an HTTP(S) proxy URL with a host and no credentials, path, query, or fragment.",
    )
}

fn provider_unavailable() -> CommandError {
    CommandError::new(
        "translation_unavailable",
        "Translation could not reach the provider. Check the network or translation proxy.",
    )
}

fn incompatible_response() -> CommandError {
    CommandError::new(
        "translation_response",
        "The translation provider returned an incompatible response.",
    )
}

fn remaining(deadline: Instant) -> Result<Duration, CommandError> {
    deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| {
            CommandError::new(
                "translation_timeout",
                "Translation timed out. Check the network or translation proxy and retry.",
            )
        })
}

fn google_translate(
    client: &reqwest::blocking::Client,
    deadline: Instant,
    text: &str,
    target: &str,
) -> Result<TranslationResult, CommandError> {
    if text.trim().is_empty() {
        return Ok(TranslationResult {
            translated_text: text.into(),
            detected_source_language: None,
        });
    }
    let mut translated = String::new();
    let mut detected = None;
    for chunk in chunks(text, CHUNK_CHARS) {
        let timeout = remaining(deadline)?;
        let response = client
            .get(GOOGLE_ENDPOINT)
            .query(&[
                ("client", "gtx"),
                ("sl", "auto"),
                ("tl", target),
                ("dt", "t"),
                ("q", chunk),
            ])
            .timeout(timeout)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|error| {
                if error.is_timeout() {
                    CommandError::new(
                        "translation_timeout",
                        "Translation timed out. Check the network or translation proxy and retry.",
                    )
                } else {
                    provider_unavailable()
                }
            })?;
        let value: serde_json::Value = response.json().map_err(|_| incompatible_response())?;
        let rows = value
            .get(0)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(incompatible_response)?;
        let chunk_start = translated.len();
        for row in rows {
            if let Some(piece) = row.get(0).and_then(serde_json::Value::as_str) {
                translated.push_str(piece);
            }
        }
        if translated.len() == chunk_start {
            return Err(incompatible_response());
        }
        detected = detected.or_else(|| {
            value
                .get(2)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
    }
    Ok(TranslationResult {
        translated_text: translated,
        detected_source_language: detected,
    })
}

fn chunks(text: &str, max_chars: usize) -> Vec<&str> {
    if text.chars().count() <= max_chars {
        return vec![text];
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut count = 0;
    for (index, character) in text.char_indices() {
        count += 1;
        if count >= max_chars && (character == '\n' || count >= max_chars + 200) {
            let end = index + character.len_utf8();
            chunks.push(&text[start..end]);
            start = end;
            count = 0;
        }
    }
    if start < text.len() {
        chunks.push(&text[start..]);
    }
    chunks
}

fn map_markdown_prose<F>(markdown: &str, mut translate: F) -> Result<String, CommandError>
where
    F: FnMut(&str) -> Result<String, CommandError>,
{
    let mut output = String::new();
    let mut fenced = false;
    let mut frontmatter = markdown.starts_with("---\n");
    for line in markdown.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            output.push_str(line);
            continue;
        }
        if frontmatter {
            output.push_str(line);
            if trimmed.trim_end() == "---" && output.len() > line.len() {
                frontmatter = false;
            }
            continue;
        }
        if fenced || line.trim().is_empty() {
            output.push_str(line);
            continue;
        }
        output.push_str(&map_inline(line, &mut translate)?);
    }
    Ok(output)
}

fn map_inline<F>(line: &str, translate: &mut F) -> Result<String, CommandError>
where
    F: FnMut(&str) -> Result<String, CommandError>,
{
    let mut output = String::new();
    let (body, ending) = line.strip_suffix("\r\n").map_or_else(
        || {
            line.strip_suffix('\n')
                .map_or((line, ""), |body| (body, "\n"))
        },
        |body| (body, "\r\n"),
    );
    let prefix = markdown_prefix(body);
    output.push_str(&body[..prefix]);
    let mut prose_start = prefix;
    let bytes = body.as_bytes();
    let mut index = prefix;
    while index < bytes.len() {
        let protected_end = if bytes[index] == b'`' {
            let ticks = bytes[index..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            body[index + ticks..]
                .find(&"`".repeat(ticks))
                .map(|offset| index + ticks + offset + ticks)
        } else if bytes[index] == b']' && bytes.get(index + 1) == Some(&b'(') {
            body[index + 2..].find(')').map(|offset| index + offset + 3)
        } else if bytes[index] == b'<' {
            body[index + 1..].find('>').map(|offset| index + offset + 2)
        } else if is_markdown_syntax(bytes[index]) {
            Some(index + 1)
        } else {
            None
        };
        if let Some(end) = protected_end {
            if prose_start < index {
                let prose = &body[prose_start..index];
                if prose.trim().is_empty() {
                    output.push_str(prose);
                } else {
                    output.push_str(&translate(prose)?);
                }
            }
            output.push_str(&body[index..end]);
            index = end;
            prose_start = end;
        } else {
            index += body[index..]
                .chars()
                .next()
                .expect("valid character boundary")
                .len_utf8();
        }
    }
    if prose_start < body.len() {
        let prose = &body[prose_start..];
        if prose.trim().is_empty() {
            output.push_str(prose);
        } else {
            output.push_str(&translate(prose)?);
        }
    }
    output.push_str(ending);
    Ok(output)
}

fn markdown_prefix(line: &str) -> usize {
    let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
    let rest = &line[indent..];
    let marker = rest
        .bytes()
        .take_while(|byte| matches!(byte, b'#' | b'>'))
        .count();
    if marker > 0
        && rest
            .as_bytes()
            .get(marker)
            .is_some_and(u8::is_ascii_whitespace)
    {
        return indent + marker + 1;
    }
    if matches!(rest.as_bytes(), [b'-' | b'+' | b'*', b' ' | b'\t', ..]) {
        return indent + 2;
    }
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0
        && matches!(rest.as_bytes().get(digits), Some(b'.' | b')'))
        && rest
            .as_bytes()
            .get(digits + 1)
            .is_some_and(u8::is_ascii_whitespace)
    {
        return indent + digits + 2;
    }
    indent
}

fn is_markdown_syntax(byte: u8) -> bool {
    matches!(
        byte,
        b'\\'
            | b'*'
            | b'_'
            | b'['
            | b']'
            | b'('
            | b')'
            | b'~'
            | b'|'
            | b'!'
            | b'#'
            | b'+'
            | b'-'
            | b'.'
            | b'>'
            | b':'
            | b'='
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_only_at_utf8_boundaries_and_preserves_order() {
        let source = "你好世界\nhello";
        assert_eq!(chunks(source, 2).concat(), source);
    }

    #[test]
    fn markdown_keeps_frontmatter_fences_inline_code_and_urls() {
        let source = "---\nname: demo\n---\n# Hello `code` [site](https://example.com)\n```rs\nlet x = 1;\n```\n";
        let result = map_markdown_prose(source, |text| Ok(text.to_uppercase())).unwrap();
        assert!(result.starts_with("---\nname: demo\n---\n"));
        assert!(result.contains("`code`"));
        assert!(result.contains("](https://example.com)"));
        assert!(result.contains("let x = 1;"));
        assert!(result.contains("# HELLO"));
    }

    #[test]
    fn markdown_structure_never_enters_the_translation_provider() {
        let source = "# Hello *world* [site](https://example.com)\n- next item\n";
        let result = map_markdown_prose(source, |text| Ok(format!("<{text}>"))).unwrap();
        assert_eq!(
            result,
            "# <Hello >*<world>* [<site>](https://example.com)\n- <next item>\n"
        );
    }

    #[test]
    fn proxy_validation_rejects_credentials_routes_and_oversized_values() {
        assert!(validate_proxy("").is_err());
        assert!(validate_proxy("socks5://127.0.0.1:1080").is_err());
        assert!(validate_proxy("http://user:secret@127.0.0.1:7890").is_err());
        assert!(validate_proxy("http://127.0.0.1:7890/path").is_err());
        assert!(validate_proxy("http://127.0.0.1:7890/").is_err());
        assert!(validate_proxy(&format!("http://{}", "a".repeat(MAX_PROXY_BYTES))).is_err());
        assert!(validate_proxy("http://127.0.0.1:7890").is_ok());
        assert!(validate_proxy("https://proxy.example:443").is_ok());
        assert!(translation_client("").is_ok());
        assert!(translation_client("http://127.0.0.1:7890").is_ok());
    }

    #[test]
    fn deadline_and_provider_errors_are_sanitized() {
        assert_eq!(
            remaining(Instant::now() - Duration::from_millis(1))
                .unwrap_err()
                .code,
            "translation_timeout"
        );
        let error = provider_unavailable();
        assert_eq!(error.code, "translation_unavailable");
        assert!(!error.message.contains("translate.googleapis.com"));
        assert!(!error.message.contains("q="));
        assert_eq!(incompatible_response().code, "translation_response");
        assert!(!incompatible_response().message.contains("Google"));
    }

    #[test]
    fn later_markdown_failure_returns_no_partial_result() {
        let mut calls = 0;
        let result = map_markdown_prose("first\nsecond\n", |text| {
            calls += 1;
            if calls == 2 {
                Err(provider_unavailable())
            } else {
                Ok(text.to_uppercase())
            }
        });
        assert_eq!(result.unwrap_err().code, "translation_unavailable");
    }
}
