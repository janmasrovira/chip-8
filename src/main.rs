#![feature(slice_as_array)]
mod architecture;
mod base;
mod cli;
mod debugger;
mod emulator;
mod language;

use architecture::*;
use clap::{Command, CommandFactory, Parser};
use clap_complete::generate;
use cli::args::{Cli, Commands};
use core::default::*;
use debugger::Debugger;
use debugger::command;
use language::*;
use ratatui::layout::*;
use ratatui::widgets::*;
use ratatui::{
    Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, List, Paragraph},
};
use std::io;
use std::io::Result;
use std::sync::mpsc;
use std::thread;

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

            let terminal = ratatui::init();
            let _result = App::new(chip).run(terminal);
            ratatui::restore();
        }
        None => {
            eprintln!("Try --help");
        }
    }
}

pub struct App {
    debugger: Debugger,
    logs: Vec<String>,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        fn v_table<'a>(d: &Debugger) -> Table<'a> {
            let widths = [Constraint::Fill(1), Constraint::Fill(1)];
            let mut rows: Vec<Row> = vec![];
            for i in 0..8 {
                let ch = &d.peek();
                rows.push(Row::new([
                    format!("V{:X?}: {:}", 2 * i, ch.rv(Register::from(2 * i))),
                    format!("V{:X?}: {:}", 2 * i + 1, ch.rv(Register::from(2 * i + 1))),
                ]))
            }
            let title: Line = Line::from("Vx registers").bold().blue().centered();
            Table::new(rows, widths).block(Block::bordered().title(title))
        }

        fn display<'a>(d: &Debugger) -> Paragraph<'a> {
            let title: Line = Line::from("Chip-8 display").bold().blue().centered();
            let text: String = d.peek().screen.to_string();
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered()
        }

        fn memory<'a>(d: &Debugger) -> List<'a> {
            let title: Line = Line::from("Memory").bold().blue().centered();
            let c = &d.peek();
            let pc: i32 = c.pc as i32;
            const H: i32 = 15 * 2;
            let mut m: Vec<String> = vec![];
            for i in (pc - H..=pc + H).step_by(2) {
                m.push(if i < 0 || i + 1 >= Chip8::MEM_SIZE as i32 {
                    "-".into()
                } else {
                    let ix = i as usize;
                    format!(
                        "{}{}",
                        RawInstr::from_bytes([c.memory[ix], c.memory[ix + 1]]),
                        if i == pc { " <--- pc" } else { "" }
                    )
                })
            }
            List::new(m).block(Block::bordered().title(title))
        }

        fn help<'a>() -> Paragraph<'a> {
            let title: Line = Line::from("Help").bold().blue().centered();
            let text: &str = "Hello!\n\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered()
        }

        let root_layout =
            Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let [display_area, tools_area] = root_layout.areas(area);
        let [other_registers, v_registers, instructions] = Layout::horizontal([
            Constraint::Percentage(100),
            Default::default(),
            Default::default(),
        ])
        .areas(tools_area);

        let p1 = display(&self.debugger);
        let p4 = memory(&self.debugger);
        let p2 = help();
        let p3 = v_table(&self.debugger);
        p1.render(display_area, buf);
        Widget::render(p4, v_registers, buf);
        p2.render(other_registers, buf);
        Widget::render(p3, instructions, buf);
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(chip: Chip8) -> Self {
        App {
            debugger: Debugger::new(chip),
            logs: Vec::new(),
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    pub fn run(mut self, mut terminal: ratatui::DefaultTerminal) -> Result<()> {
        let (sender, receiver) = mpsc::channel::<command::Command>();
        thread::spawn(move || {
            Self::input_loop(sender);
        });
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            match receiver.recv().expect("receiver failed") {
                command::Command::Exit => break,
                command::Command::StepForward => {
                    self.debugger.step_forward();
                    self.logs.push(String::from("step"));
                }
                command::Command::StepBackward => {
                    if self.debugger.step_back() {
                    } else {
                        // TODO
                    }
                }
            }
        }
        Ok(())
    }

    pub fn input_loop(sender: mpsc::Sender<command::Command>) {
        loop {
            match crossterm::event::read() {
                Ok(e) => {
                    if let Some(c) = command::Command::command_from_event(e) {
                        sender.send(c).expect("sender failed");
                    }
                }
                Err(_) => panic!("input error"),
            }
        }
    }
}
