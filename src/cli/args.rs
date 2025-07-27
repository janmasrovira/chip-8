use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::*;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },

    ///Run the CHIP-8 emulator
    #[command(about = format!("Run the CHIP-8 emulator"))]
    Run {
        #[arg()]
        file: PathBuf,
    },
}
