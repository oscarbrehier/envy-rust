use clap::{Parser, Subcommand};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser)]
#[command(name = "envy", version, about = "Format and validate .env files")]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Format { path: String },
    Validate { 
        path: String,
        #[arg(short, long, default_value = "keep-first")]
        dupes: String 
    },
}

fn parse_env_file(path: &str) -> Result<Vec<(String, String)>, std::io::Error> {

    let content = fs::read_to_string(path).expect("Could not read file");
    let mut data: Vec<(String, String)> = Vec::new();

    for line in content.lines().map(|l| l.trim_end_matches(&['\r', '\n'][..])) {

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue ;
        }

        if trimmed.starts_with('#') {
            continue ;
        }
    
        if let Some((key, value)) = trimmed.split_once('=') {

            let key = key.trim();
            let value = value.trim();

            data.push((key.to_string(), value.to_string()));
            
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

fn validate(path: &str, dupe_strategy: &str) {

    let data: Vec<(String, String)> = parse_env_file(path).expect("parsing failed");

    for (key, value) in &data {
        println!("Key: {}, Value: {}", key, value);
    }

    println!("{}", dupe_strategy);

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
        Commands::Validate { path, dupes } => {
            
            validate(&path, &dupes);

        }
    }

    // let mut file = File::open(".env")?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;
    // println!("{}", contents);
    // Ok(())
}
