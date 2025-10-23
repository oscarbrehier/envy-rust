mod cli;
mod parser;
mod validator;
mod fu;
mod formatters;
mod sorter;

use cli::{Cli, Commands};
use clap::Parser;

fn main() {

    let cli = Cli::parse();

    match cli.command {

        Commands::Format { path, dupes, dry_run, strip_exports } => {

            println!("\nFormatting file: {}\n", path);

            let formatted_file = formatters::format_env_file(&path, &dupes, dry_run, strip_exports).expect("formatting failed");

            if !dry_run {
                fu::write_to_file(&path, &formatted_file).expect("failed to write");
            }

        }
        Commands::Validate { path, check_required, error } => {

            if !validator::validate(&path, check_required, error) {
                std::process::exit(1);
            }

        }

        Commands::Sort { path, method, .. } => {

            let sorted_content = sorter::sort(&path, &method).expect("sorting failed");
            fu::write_to_file(&path, &sorted_content).expect("failed to write");


        }

    }

}