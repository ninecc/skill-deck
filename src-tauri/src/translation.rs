use crate::{
    cli::{CliManager, CommandError},
    preview,
};
use serde::Serialize;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

const GOOGLE_ENDPOINT: &str = "https://translate.googleapis.com/translate_a/single";
const CHUNK_CHARS: usize = 3_500;
const MAX_PROXY_BYTES: usize = 2_048;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(15);
const BATCH_WORKERS: usize = 4;
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
    let result = if markdown {
        translate_markdown(&text, BATCH_WORKERS, |batch| {
            google_translate(&client, deadline, batch, provider_target)
        })?
    } else {
        google_translate(&client, deadline, &text, provider_target)?
    };
    Ok(TranslationResult {
        translated_text: result.translated_text,
        detected_source_language: result.detected_source_language,
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

fn background_failure() -> CommandError {
    CommandError::new("internal", "The background translation could not complete.")
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

#[derive(Debug)]
struct MarkdownBatch {
    payload: String,
    segments: Vec<(usize, std::ops::Range<usize>)>,
}

fn markdown_batches(markdown: &str) -> Result<Vec<MarkdownBatch>, CommandError> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    map_markdown_prose(markdown, |prose| {
        let start = (prose.as_ptr() as usize)
            .checked_sub(markdown.as_ptr() as usize)
            .filter(|start| *start >= cursor)
            .filter(|start| markdown.get(*start..*start + prose.len()) == Some(prose))
            .ok_or_else(background_failure)?;
        ranges.push(start..start + prose.len());
        cursor = start + prose.len();
        Ok(prose.to_owned())
    })?;

    let mut segments = Vec::new();
    for range in ranges {
        let mut start = range.start;
        while start < range.end {
            let id = segments.len();
            let overhead = span_open(id).chars().count() + "</span>".len();
            let mut encoded_chars = 0;
            let mut end = start;
            for (offset, character) in markdown[start..range.end].char_indices() {
                let next = escaped_char(character).chars().count();
                if overhead + encoded_chars + next > CHUNK_CHARS {
                    break;
                }
                encoded_chars += next;
                end = start + offset + character.len_utf8();
            }
            if end == start {
                return Err(background_failure());
            }
            segments.push((id, start..end));
            start = end;
        }
    }

    let mut batches: Vec<MarkdownBatch> = Vec::new();
    for (id, range) in segments {
        let entry = format!(
            "{}{}</span>",
            span_open(id),
            escape_html(&markdown[range.clone()])
        );
        let entry_chars = entry.chars().count();
        if batches
            .last()
            .is_none_or(|batch| batch.payload.chars().count() + entry_chars > CHUNK_CHARS)
        {
            batches.push(MarkdownBatch {
                payload: String::new(),
                segments: Vec::new(),
            });
        }
        let batch = batches.last_mut().expect("a batch was just created");
        batch.payload.push_str(&entry);
        batch.segments.push((id, range));
    }
    Ok(batches)
}

fn span_open(id: usize) -> String {
    format!(r#"<span data-sd="{id}">"#)
}

fn escaped_char(character: char) -> String {
    match character {
        '&' => "&amp;".into(),
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        '"' => "&quot;".into(),
        '\'' => "&#39;".into(),
        value => value.into(),
    }
}

fn escape_html(text: &str) -> String {
    text.chars().map(escaped_char).collect()
}

fn decode_html(text: &str) -> Result<String, CommandError> {
    let mut decoded = String::new();
    let mut rest = text;
    while let Some(index) = rest.find(['&', '<', '>']) {
        decoded.push_str(&rest[..index]);
        rest = &rest[index..];
        let (entity, value) = [
            ("&amp;", '&'),
            ("&lt;", '<'),
            ("&gt;", '>'),
            ("&quot;", '"'),
            ("&#39;", '\''),
        ]
        .into_iter()
        .find(|(entity, _)| rest.starts_with(entity))
        .ok_or_else(incompatible_response)?;
        decoded.push(value);
        rest = &rest[entity.len()..];
    }
    decoded.push_str(rest);
    Ok(decoded)
}

fn parse_batch(text: &str, expected: &[usize]) -> Result<Vec<String>, CommandError> {
    let mut rest = text;
    let mut translated = Vec::with_capacity(expected.len());
    for id in expected {
        rest = rest
            .strip_prefix(&span_open(*id))
            .ok_or_else(incompatible_response)?;
        let end = rest.find("</span>").ok_or_else(incompatible_response)?;
        translated.push(decode_html(&rest[..end])?);
        rest = &rest[end + "</span>".len()..];
    }
    if !rest.is_empty() {
        return Err(incompatible_response());
    }
    Ok(translated)
}

fn translate_markdown<F>(
    markdown: &str,
    workers: usize,
    translate: F,
) -> Result<TranslationResult, CommandError>
where
    F: Fn(&str) -> Result<TranslationResult, CommandError> + Sync,
{
    let batches = markdown_batches(markdown)?;
    if batches.is_empty() {
        return Ok(TranslationResult {
            translated_text: markdown.to_owned(),
            detected_source_language: None,
        });
    }

    let next = AtomicUsize::new(0);
    let (sender, receiver) = std::sync::mpsc::channel();
    let workers_completed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::thread::scope(|scope| {
            for _ in 0..workers.min(batches.len()).max(1) {
                let sender = sender.clone();
                let batches = &batches;
                let translate = &translate;
                let next = &next;
                scope.spawn(move || loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(batch) = batches.get(index) else {
                        break;
                    };
                    if sender.send((index, translate(&batch.payload))).is_err() {
                        break;
                    }
                });
            }
            drop(sender);
        });
    }));
    if workers_completed.is_err() {
        return Err(background_failure());
    }
    let mut translated: Vec<_> = receiver.into_iter().collect();
    translated.sort_by_key(|(index, _)| *index);
    if translated.len() != batches.len() {
        return Err(background_failure());
    }

    let mut output = String::with_capacity(markdown.len());
    let mut detected = None;
    let mut source_cursor = 0;
    for (batch, (_, result)) in batches.into_iter().zip(translated) {
        let result = result?;
        let ids: Vec<_> = batch.segments.iter().map(|(id, _)| *id).collect();
        let bodies = parse_batch(&result.translated_text, &ids)?;
        for ((_, range), body) in batch.segments.into_iter().zip(bodies) {
            output.push_str(&markdown[source_cursor..range.start]);
            output.push_str(&body);
            source_cursor = range.end;
        }
        detected = detected.or(result.detected_source_language);
    }
    output.push_str(&markdown[source_cursor..]);
    Ok(TranslationResult {
        translated_text: output,
        detected_source_language: detected,
    })
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
    use std::sync::atomic::AtomicUsize;

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

    #[test]
    fn markdown_translation_is_bounded_ordered_and_atomic() {
        let source = (0..12)
            .map(|index| format!("fragment-{index} {}\n", "x".repeat(1_800)))
            .collect::<String>();
        let batch_count = markdown_batches(&source).unwrap().len();
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let result = translate_markdown(&source, 3, |text| {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(current, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(2));
            active.fetch_sub(1, Ordering::SeqCst);
            Ok(TranslationResult {
                translated_text: text.replace("fragment", "FRAGMENT"),
                detected_source_language: Some("en".into()),
            })
        })
        .unwrap();
        assert_eq!(
            result.translated_text,
            source.replace("fragment", "FRAGMENT")
        );
        assert!((2..=3).contains(&peak.load(Ordering::SeqCst)));

        let completed = AtomicUsize::new(0);
        let result = translate_markdown(&source, 3, |text| {
            let call = completed.fetch_add(1, Ordering::SeqCst);
            if call == 1 {
                return Err(provider_unavailable());
            }
            Ok(TranslationResult {
                translated_text: text.into(),
                detected_source_language: None,
            })
        });
        assert_eq!(result.unwrap_err().code, "translation_unavailable");
        assert_eq!(completed.load(Ordering::SeqCst), batch_count);
    }

    #[test]
    fn markdown_batches_escape_pack_and_strictly_validate_markers() {
        let escaped = escape_html("&<>\"'");
        assert_eq!(escaped, "&amp;&lt;&gt;&quot;&#39;");
        assert_eq!(decode_html(&escaped).unwrap(), "&<>\"'");
        assert!(decode_html("&copy;").is_err());

        let source = format!("first & <tag>\n{}\n", "&".repeat(CHUNK_CHARS));
        let batches = markdown_batches(&source).unwrap();
        assert!(batches.len() > 1);
        assert!(batches
            .iter()
            .all(|batch| batch.payload.chars().count() <= CHUNK_CHARS));
        let first_ids: Vec<_> = batches[0].segments.iter().map(|(id, _)| *id).collect();
        assert!(parse_batch(&batches[0].payload, &first_ids).is_ok());
        assert!(parse_batch(
            &batches[0]
                .payload
                .replacen("data-sd=\"0\"", "data-sd=\"9\"", 1),
            &first_ids,
        )
        .is_err());
        assert!(parse_batch(
            &format!("{}<span data-sd=\"0\">duplicate</span>", batches[0].payload),
            &first_ids,
        )
        .is_err());
        assert!(parse_batch("", &first_ids).is_err());
        assert!(parse_batch("<span data-sd=\"999\">text</span>", &first_ids).is_err());
        assert!(parse_batch("<span data-sd=\"0\"><b>text</b></span>", &[0]).is_err());
        assert!(parse_batch("<span data-sd=\"0\">&copy;</span>", &[0]).is_err());

        let unicode = format!("{}\n", "界".repeat(CHUNK_CHARS + 1));
        let unicode_batches = markdown_batches(&unicode).unwrap();
        assert!(unicode_batches.len() > 1);
        assert!(unicode_batches
            .iter()
            .all(|batch| batch.payload.chars().count() <= CHUNK_CHARS));
        let decoded = unicode_batches
            .iter()
            .flat_map(|batch| {
                let ids: Vec<_> = batch.segments.iter().map(|(id, _)| *id).collect();
                parse_batch(&batch.payload, &ids).unwrap()
            })
            .collect::<String>();
        assert_eq!(decoded, unicode.trim_end());
    }

    #[test]
    fn markdown_detection_uses_document_order_not_completion_order() {
        let source = format!(
            "first {}\nsecond {}\n",
            "x".repeat(2_000),
            "y".repeat(2_000)
        );
        let result = translate_markdown(&source, 2, |payload| {
            let first = payload.starts_with("<span data-sd=\"0\">");
            if first {
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(TranslationResult {
                translated_text: payload.into(),
                detected_source_language: Some(if first { "first" } else { "later" }.into()),
            })
        })
        .unwrap();
        assert_eq!(result.translated_text, source);
        assert_eq!(result.detected_source_language.as_deref(), Some("first"));
    }

    #[test]
    fn markdown_translation_uses_source_ranges_and_contains_worker_panics() {
        let source = "`repeat` repeat\n";
        let result = translate_markdown(source, 2, |text| {
            Ok(TranslationResult {
                translated_text: text.replace("repeat", "REPEAT"),
                detected_source_language: None,
            })
        })
        .unwrap();
        assert_eq!(result.translated_text, "`repeat` REPEAT\n");

        let result = translate_markdown("first\nsecond\n", 2, |text| {
            if text.contains("second") {
                panic!("provider worker panic");
            }
            Ok(TranslationResult {
                translated_text: text.into(),
                detected_source_language: None,
            })
        });
        assert_eq!(result.unwrap_err().code, "internal");
    }
}
