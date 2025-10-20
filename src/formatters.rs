use crate::parser::{Line, parse_env_file};

pub fn dry_run_action(width: usize, line: usize, old: Option<&str>, new: Option<&str>, note: &str) -> String {

    let mut msg = String::new();

    if let Some(old) = old {
        msg.push_str(&format!("{:>width$} | - {}\n", line, old, width = width));
    }
    if let Some(new) = new {
        msg.push_str(&format!("{:>width$} | + {}\n", "", new, width = width));
    }
    if !note.is_empty() {
        msg.push_str(&format!("{:>width$} | ({})\n", "", note, width = width));
    }

    msg

}

pub fn format_env_file(path: &str, dupes: &str, dry_run: bool) -> Result<String, std::io::Error> {

    let mut lines = parse_env_file(path).expect("failed to parse file");
    let mut formatted: Vec<String> = Vec::new();
    let mut dry_run_changes: Vec<(usize, String)> = Vec::new();
    let mut keys: Vec<String> = Vec::new();

    let mut reformatted_count = 0;
    let mut duplicate_count = 0;
    let mut invalid_count = 0;

    let reverse = dupes == "keep-last";
    if reverse {
        lines.reverse();
    }

    let lines_num = lines.iter().enumerate().len();
    let width = lines_num.to_string().len();

    for line in lines {

        match line {

            Line::Comment(text) => formatted.push(format!("{}\n", text)),
            Line::Empty => formatted.push(String::from('\n')),
            Line::KeyValue { key, value, line, inline_comment, has_export, .. } => {

                let normalized_key = if has_export == Some(true) {

                    let no_export_key = key.strip_prefix("export ").unwrap_or(&key);

                    if dry_run {

                        let msg = dry_run_action(
                            width,
                            line,
                            Some(&format!("export {}={}", key, value)),
                            Some(&format!("{}={}", no_export_key, value)),
                            "Removed export keyword",
                        );

                        dry_run_changes.push((line, msg));

                    }

                    no_export_key.to_string()

                } else {
                    key.clone()
                };

                if keys.contains(&normalized_key) {

                    if dry_run {
                        
                        let msg = dry_run_action(
                            width,
                            line,
                            Some(&format!("{}={}", normalized_key, value)),
                            Some("(line removed - duplicate key"),
                            ""
                        );

                        dry_run_changes.push((line, msg));

                    } else {
                        eprintln!("Removed duplicate key: {} (line {})", normalized_key, line);
                    }

                    duplicate_count += 1;

                    continue ;

                }

                if normalized_key.contains(char::is_whitespace) || (value.contains(char::is_whitespace) && !inline_comment.is_some()) {

                    if dry_run {

                        let msg = dry_run_action(width, line, 
                            Some(&format!("{}={}", normalized_key, value)), 
                            Some(&format!("{}={}", normalized_key.trim(), value.trim())), 
                            "removed extra spaces"
                        );
                        
                        dry_run_changes.push((line, msg));

                    } else {

                        formatted.push(format!("{}={}\n", normalized_key.trim(), value.trim()));
                        reformatted_count += 1;

                        keys.push(normalized_key.trim().to_string());

                    }

                    continue ;

                }

                if let Some(comment) = inline_comment {
                    formatted.push(format!("{}={} {}\n", normalized_key, value, comment));
                } else {
                    formatted.push(format!("{}={}\n", normalized_key, value));
                }

                reformatted_count += 1;

                keys.push(normalized_key);

            },
            Line::Invalid { content, line } => {

                if dry_run {
                    
                    let msg = dry_run_action(width, line, 
                        Some("INVALID LINE"), 
                        Some("(line removed - invalid syntax)"), 
                        ""
                    );

                    dry_run_changes.push((line, msg));
                    
                    continue ;

                } else {

                    eprintln!("Skipping invalid line: {} (line {})", content, line);
                    invalid_count += 1;

                }

            }

        }

    }
    
    if reverse {
        dry_run_changes.sort_by_key(|(line, _)| *line);
        formatted.reverse();
    }

    if dry_run {
        for (_, msg) in dry_run_changes {
            println!("{}", msg);
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("Summary:");
    println!(" • {} lines reformatted", reformatted_count);
    println!(" • {} duplicate keys removed ({})", duplicate_count, dupes);
    println!(" • {} invalid lines removed\n", invalid_count);

    let formatted_content = formatted
        .join("");

    Ok(formatted_content)

}