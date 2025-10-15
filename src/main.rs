use clap::{Parser, Subcommand};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "envy", version, about = "Format and validate .env files")]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

struct KeyInfo {
    value: String,
    line: usize,
}

#[derive(Subcommand)]
enum Commands {
    Format { path: String },
    Validate { 
        path: String,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        error: bool,
        // #[arg(short, long, default_value = "keep-first")]
        // dupes: String 
    },
}

fn parse_env_file(path: &str) -> Result<HashMap<String, Vec<KeyInfo>>, std::io::Error> {

    let content = fs::read_to_string(path).expect("Could not read file");
    let mut data: HashMap<String, Vec<KeyInfo>> = HashMap::new();
    let mut line_idx: usize = 0;

    for line in content.lines().map(|l| l.trim_end_matches(&['\r', '\n'][..])) {

        line_idx += 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue ;
        }

        if trimmed.starts_with('#') {
            continue ;
        }

        // println!("line {} idx {}\n", line, line_idx);
        
        if let Some((key, value)) = trimmed.split_once('=') {
            
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            // println!("value {} key {} at idx {}\n", key, value, line_idx);
            
            let info = KeyInfo { value, line: line_idx };

            data.entry(key).or_insert_with(Vec::new).push(info);
            
        }

    }

    Ok(data)

}

fn format_env_file(path: &str) -> Result<String, std::io::Error> {

    let content = fs::read_to_string(path).expect("Could not read file");
   
    let mut formatted_content = String::from("");
    let mut formatted_lines = 0;
    let mut skipped_invalid  = 0;

    // let mut keys: Vec<String> = Vec::new();

    for line in content.lines().map(|l| l.trim_end_matches(&['\r', '\n'][..])) {
        
        let trimmed = line.trim();

        if trimmed.is_empty() {
            formatted_content.push('\n');
            continue ;
        }

        if trimmed.starts_with('#') {
            formatted_content.push_str(trimmed);
            formatted_content.push('\n');
            continue ;
        }

        // println!("Line {:?}:", line.chars().nth(1).unwrap();
    
        if let Some((key, value)) = trimmed.split_once('=') {

            let key = key.trim();
            let value = value.trim();

            // if keys.contains(&key.to_string()) {
            //     println!("Skipping key {} found duplicate. Value: {}\n", key, value);
            //     continue ;
            // }

            // keys.push(String::from(key));

            formatted_content.push_str(&format!("{}={}\n", key, value));
            formatted_lines += 1;

        } else {
            skipped_invalid += 1;
        }

    }
    
    println!(
        "\n✅ Formatting complete: {} lines formatted, {} invalid lines skipped.",
        formatted_lines, skipped_invalid
    );

    Ok(formatted_content)
    
}

fn validate(path: &str, error_mode: bool) {

    let data: HashMap<String, Vec<KeyInfo>> = parse_env_file(path).expect("parsing failed");
    let mut has_error: bool = false;

    for (key, infos) in &data {

        // println!("key {} values {}", key, infos.iter().map(|info| info.value.to_string()).collect::<Vec<String>>().join("|"));

        if infos.len() > 1 {

            has_error = true;

            let lines: String = infos
                .iter()
                .map(|info| info.line.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            println!(
                "Duplicate keys found: `{}` (lines {})",
                key, lines
            );

        }

        for info in infos {

            if key.trim().is_empty() {
                has_error = true;
                println!("Empty key found at line {}", info.line);
            }

            if info.value.trim().is_empty() {
                has_error = true;
                println!("Empty value found for key: {} (line {})", key, info.line);
            }

        }

    }

    if error_mode && has_error {
        std::process::exit(1);
    }

}

fn main() {

    let cli = Cli::parse();

    match cli.command {
        Commands::Format { path } => {

            println!("Formatting file: {}", path);

            let formatted_file = format_env_file(&path)
                .expect("formatting failed");

            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)
                .expect("cannot open file");

            file
                .write(formatted_file.as_bytes())
                .expect("write failed");

        }
        Commands::Validate { path, error } => {
            
            validate(&path, error);

        }
    }

    // let mut file = File::open(".env")?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;
    // println!("{}", contents);
    // Ok(())
}
