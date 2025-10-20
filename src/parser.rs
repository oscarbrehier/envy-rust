use std::fs;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum Line {
    Comment(String),
    Empty,
    KeyValue {
        key: String,
        value: String,
        line: usize,
        inline_comment: Option<String>,
        references: Option<Vec<String>>,
        has_export: Option<bool>
    },
    Invalid {
        content: String,
        line: usize,
    },
}

fn extract_variable_reference(value: &str) -> Vec<String> {

    let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();

    re.captures_iter(value)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()

}

pub fn parse_inline_comment(line: &str) -> (&str, Option<&str>) {

    let mut in_quotes = false;
    let mut quote_char = ' ';
    let mut escaped = false;

    for (i, ch) in line.chars().enumerate() {

        if escaped {
            escaped = false;
            continue ;
        }

        if ch == '\\' {
            escaped = true;
            continue ;
        }

        if ch == '"' || ch == '\'' {
            
            if !in_quotes {
                in_quotes = true;
                quote_char = ch;
            } else if ch == quote_char {
                in_quotes = false;
            }

        }

        if ch == '#' && !in_quotes {

            let content = line[..i].trim_end();
            let comment = line[i..].trim();
            return (content, Some(comment));

        }

    }

    (line, None)

}

pub fn parse_env_file(path: &str) -> Result<Vec<Line>, std::io::Error> {
    
    let content = fs::read_to_string(path)?;
    
    let mut lines = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {

        let line_num = idx + 1;
        let trimmed = raw_line.trim_end_matches(&['\r', '\n'][..]).trim();

        if trimmed.is_empty() {
            lines.push(Line::Empty);
            continue;
        }

        if trimmed.starts_with('#') {
            lines.push(Line::Comment(trimmed.to_string()));
            continue;
        }

        let (content, inline_comment) = parse_inline_comment(trimmed);

        if let Some((key, value)) = content.split_once('=') {

            let key = key.to_string();
            let value = value.to_string();

            if key.trim().is_empty() || (value.trim().is_empty() && !inline_comment.is_some()) {
                lines.push(Line::Invalid { content: trimmed.to_string(), line: line_num });
                continue ;
            }

            let references = extract_variable_reference(content);

            let has_export = trimmed.starts_with("export ");

            lines.push(Line::KeyValue {
                key,
                value,
                line: line_num,
                inline_comment: inline_comment.map(|s| s.to_string()),
                references: Some(references),
                has_export: Some(has_export)
            });

        } else {

            lines.push(Line::Invalid {
                content: trimmed.to_string(),
                line: line_num,
            });

        }

    }

    Ok(lines)

}

pub fn get_keys(parsed_lines: &[Line]) -> Vec<String> {

    parsed_lines
        .iter()
        .filter_map(|line| {

            if let Line::KeyValue { key, .. } = line {
                Some(key.clone())
            } else {
                None
            }

        })
        .collect()

}