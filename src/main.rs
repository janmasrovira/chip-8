#![feature(slice_as_chunks)]
#![feature(slice_as_array)]
#![feature(slice_patterns)]
mod architecture;
mod base;
mod cli;
mod emulator;
mod language;
mod parser;

use architecture::Chip8;
use clap::{Command, CommandFactory, Parser};
use clap_complete::generate;
use cli::args::{Cli, Commands};
use std::io;

fn main() {
    let cli: Cli = Cli::parse();

    match &cli.command {
        Some(Commands::Completions { shell }) => {
            let mut cmd: Command = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(*shell, &mut cmd, bin_name, &mut io::stdout());
        }
        Some(Commands::Run { file }) => {
            println!("Beep Boop, I'm CHIP-8 and I'll run {}", file.display());
            let mut chip = Chip8::new();
            chip.load_memory(file)
                .expect("Failed to load file from memory");
            chip.run();
        }
        None => {
            eprintln!("Try --help");
        }
    }
}
