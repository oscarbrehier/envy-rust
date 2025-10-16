use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

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
        dupes: String
    },
    Validate {
        path: String,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        error: bool,
    },
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

        if let Some((key, value)) = trimmed.split_once('=') {

            let key = key.trim().to_string();
            let value = value.trim().to_string();

            if key.is_empty() || value.is_empty() {
                lines.push(Line::Invalid { content: trimmed.to_string(), line: line_num });
                continue ;
            }

            lines.push(Line::KeyValue {
                key,
                value,
                line: line_num,
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

fn format_env_file(path: &str, dupes: &str) -> Result<String, std::io::Error> {

    let mut lines = parse_env_file(path).expect("failed to parse file");
    let mut formatted: Vec<String> = Vec::new();

    let mut keys: Vec<String> = Vec::new();
    let mut formatted_lines = 0;
    let mut skipped_invalid = 0;

    if dupes == "keep-last" {
        lines.reverse();
    }

    for line in lines {

        match line {

            Line::Comment(text) => formatted.push(format!("{}\n", text)),
            Line::Empty => formatted.push(String::from('\n')),
            Line::KeyValue { key, value, line } => {

                if keys.contains(&key) {

                    eprintln!("Removed duplicate key: {} (line {})", key, line);
                    continue ;

                }

                formatted.push(format!("{}={}\n", key, value));
                formatted_lines += 1;

                keys.push(key);

            },
            Line::Invalid { content, line } => {
                eprintln!("Skipping invalid line: {} (line {})", content, line);
                skipped_invalid += 1;
            }

        }

    }

    if dupes == "keep-last" {
        formatted.reverse();
    }

    println!(
        "\n✅ Formatting complete: {} lines formatted, {} invalid lines skipped.",
        formatted_lines, skipped_invalid
    );

    let formatted_content = formatted
        .join("");

    Ok(formatted_content)

}

fn validate(path: &str, error_mode: bool) -> bool {

    let lines = parse_env_file(path).expect("failed to parse file");
    let mut map: HashMap<String, Vec<(String, usize)>> = HashMap::new();
    let mut has_error = false;

    for line in &lines {

        match line {

            Line::KeyValue { key, value, line } => {
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

fn main() {
    let cli = Cli::parse();

    match cli.command {

        Commands::Format { path, dupes } => {

            println!("Formatting file: {}", path);

            let formatted_file = format_env_file(&path, &dupes).expect("formatting failed");

            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)
                .expect("cannot open file");

            file.write(formatted_file.as_bytes()).expect("write failed");

        }
        Commands::Validate { path, error } => {

            if !validate(&path, error) {
                std::process::exit(1);
            }

        }

    }

    // let mut file = File::open(".env")?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;
    // println!("{}", contents);
    // Ok(())
}
