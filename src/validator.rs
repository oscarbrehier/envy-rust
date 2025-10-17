use crate::parser::{Line, parse_env_file, get_keys};
use std::collections::HashMap;

pub fn check_required_keys(parsed_file: &Vec<Line>) {

    let example_lines = parse_env_file(".env.example").expect("failed to parse file");

    let required_keys: Vec<String> = get_keys(&example_lines);
    let keys: Vec<String> = get_keys(&parsed_file);

    for rkey in required_keys {

        if !keys.contains(&rkey) {
            println!("Missing required key {}", rkey);
        }

    }

}

pub fn validate(path: &str, check_required: bool, error_mode: bool) -> bool {

    let lines = parse_env_file(path).expect("failed to parse file");

    let mut map: HashMap<String, Vec<(String, usize)>> = HashMap::new();
    let mut has_error = false;
    
    if check_required {
        check_required_keys(&lines);
    }

    for line in &lines {

        match line {

            Line::KeyValue { key, value, line, .. } => {
                map.entry(key.clone())
                    .or_insert_with(Vec::new)
                    .push((value.clone(), *line));
            }
            _ => {}

        }

    }

    for (key, entries) in map {

        if entries.len() > 1 {

            has_error = true;

            let lines = entries
                .iter()
                .map(|(_, l)| l.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            println!("Duplicate key: `{}` (lines {})", key, lines);

        }

        for (value, line) in entries {

            if key.trim().is_empty() {
                has_error = true;
                println!("Empty key found at line {}", line);
            }

            if value.trim().is_empty() {
                has_error = true;
                println!("Empty value for key `{}` (line {})", key, line);
            }

        }

    }

    if has_error && error_mode {
        return false;
    }

    if !has_error {
        println!("Validation passed: no issues found");
    }

    true

}
