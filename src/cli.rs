use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "envy", version, about = "Format and validate .env files")]

pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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