use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;

#[derive(Parser)]
#[command(name = "envy", version, about = "Format and validate .env files")]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone)]
enum Line {
    Comment(String),
    Empty,
    KeyValue {
        key: String,
        value: String,
        line: usize,
        inline_comment: Option<String>
    },
    Invalid {
        content: String,
        line: usize,
    },
}

#[derive(Subcommand)]
enum Commands {
    Format {
        path: String,
        #[arg(short, long, default_value = "keep-first")]
        dupes: String,
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    Sort {
        path: String,
        #[arg(short, long, default_value = "group")]
        method: String,
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    Validate {
        path: String,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        check_required: bool,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        error: bool,
    },
}

fn parse_inline_comment(line: &str) -> (&str, Option<&str>) {

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

fn parse_env_file(path: &str) -> Result<Vec<Line>, std::io::Error> {
    
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

            if key.trim().is_empty() || value.trim().is_empty() {
                lines.push(Line::Invalid { content: trimmed.to_string(), line: line_num });
                continue ;
            }

            lines.push(Line::KeyValue {
                key,
                value,
                line: line_num,
                inline_comment: inline_comment.map(|s| s.to_string())
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

fn get_keys(parsed_lines: &[Line]) -> Vec<String> {

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

fn dry_run_action(width: usize, line: usize, old: Option<&str>, new: Option<&str>, note: &str) -> String {

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

fn format_env_file(path: &str, dupes: &str, dry_run: bool) -> Result<String, std::io::Error> {

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
            Line::KeyValue { key, value, line, inline_comment } => {

                if keys.contains(&key) {

                    if dry_run {
                        
                        let msg = dry_run_action(
                            width,
                            line,
                            Some(&format!("{}={}", key, value)),
                            Some("(line removed - duplicate key"),
                            ""
                        );

                        dry_run_changes.push((line, msg));

                    } else {
                        eprintln!("Removed duplicate key: {} (line {})", key, line);
                    }

                    duplicate_count += 1;

                    continue ;

                }

                if key.contains(char::is_whitespace) || (value.contains(char::is_whitespace) && !inline_comment.is_some()) {

                    if dry_run {

                        let msg = dry_run_action(width, line, 
                            Some(&format!("{}={}", key, value)), 
                            Some(&format!("{}={}", key.trim(), value.trim())), 
                            "removed extra spaces"
                        );
                        
                        dry_run_changes.push((line, msg));

                    } else {

                        formatted.push(format!("{}={}\n", key.trim(), value.trim()));
                        reformatted_count += 1;

                        keys.push(key.trim().to_string());

                    }

                    continue ;

                }

                if let Some(comment) = inline_comment {
                    formatted.push(format!("{}={} {}\n", key, value, comment));
                } else {
                    formatted.push(format!("{}={}\n", key, value));
                }

                reformatted_count += 1;

                keys.push(key);

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

fn check_required_keys(parsed_file: &Vec<Line>) {

    let example_lines = parse_env_file(".env.example").expect("failed to parse file");

    let required_keys: Vec<String> = get_keys(&example_lines);
    let keys: Vec<String> = get_keys(&parsed_file);

    for rkey in required_keys {

        if !keys.contains(&rkey) {
            println!("Missing required key {}", rkey);
        }

    }

}

fn validate(path: &str, check_required: bool, error_mode: bool) -> bool {

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

fn sort(path: &str, method: &str) -> Result<String, std::io::Error> {

    let lines: Vec<Line> = parse_env_file(path).expect("failed to parse file");

    let mut output = String::new();
   
    if method == "alpha" {

        let mut key_list = Vec::<Vec<String>>::new();

        for line in &lines {

            if let Line::KeyValue { key, value, .. } = line {
                key_list.push(vec![key.to_string(), value.to_string()]);
            }

        }

        key_list.sort_by(|a, b| a[0].cmp(&b[0]));
        
        for kv in key_list {
            output.push_str(&format!("{}={}\n", kv[0], kv[1]));
        }

        output.push('\n');
        
    } else {

        let mut grouped_keys: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for line in &lines {

            if let Line::KeyValue { key, value, .. } = line {

                let prefix = if key.split('_').nth(1).is_none() {
                    String::from("MISC")
                } else {
                    key.split('_').next().unwrap_or("").to_string()
                };

                grouped_keys.entry(prefix)
                    .or_insert_with(Vec::new)
                    .push((key.clone(), value.clone()));

            }

        }

        for values in grouped_keys.values_mut() {
            values.sort_by(|a, b| a.0.cmp(&b.0));
        }
        
        for (prefix, entries) in grouped_keys {

            output.push_str(&format!("# {}\n", prefix.to_uppercase()));

            for (key, value) in entries {
                output.push_str(&format!("{}={}\n", key, value));
            }

            output.push('\n');

        }

    }

    Ok(output)

}

fn backup_file(path: &str) -> Result<(), std::io::Error> {

    let content = fs::read_to_string(path)?;
    let backup_path = format!("{}.bak", path);

    fs::write(&backup_path, content)?;

    println!("Backup of {} located at {}", path, &backup_path);

    Ok(())

}

fn write_to_file(path: &str, content: &str) -> Result<(), std::io::Error> {
    
    let temp_path = format!("{}.tmp", path);

    backup_file(path).expect("env backup failed");
    fs::write(&temp_path, content)?;
    fs::rename(&temp_path, path)?;

    Ok(())

}

fn main() {
    let cli = Cli::parse();

    match cli.command {

        Commands::Format { path, dupes, dry_run } => {

            println!("\nFormatting file: {}\n", path);

            let formatted_file = format_env_file(&path, &dupes, dry_run).expect("formatting failed");

            if !dry_run {
                write_to_file(&path, &formatted_file).expect("failed to write");
            }

        }
        Commands::Validate { path, check_required, error } => {

            if !validate(&path, check_required, error) {
                std::process::exit(1);
            }

        }

        Commands::Sort { path, method, .. } => {

            let sorted_content = sort(&path, &method).expect("sorting failed");
            write_to_file(&path, &sorted_content).expect("failed to write");


        }

    }

    // let mut file = File::open(".env")?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;
    // println!("{}", contents);
    // Ok(())
}
