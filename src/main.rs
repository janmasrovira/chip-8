mod cli;

use clap::{Command, CommandFactory, Parser};
use clap_complete::generate;
use cli::args::{Cli, Commands};
use std::io;

fn main() {
    let cli : Cli = Cli::parse();

    match &cli.command {
        Some(Commands::Completions { shell }) => {
            let mut cmd: Command = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(*shell, &mut cmd, bin_name, &mut io::stdout());
        }
        Some(Commands::Run { file }) => {
            println!("Beep Boop, I'm CHIP-8 and I'll run {}", file.display());
        }
        None => {
            eprintln!("Try --help");
        }
    }
}
