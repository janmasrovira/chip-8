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
use language::*;
use ratatui::layout::*;
use ratatui::text::*;
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
            let ch = &d.peek();
            rows.push(Row::new([format!("I: {}", ch.i)]));
            for i in 0..8 {
                rows.push(Row::new([
                    format!("V{:X?}: {}", 2 * i, ch.rv(Register::from(2 * i))),
                    format!("V{:X?}: {}", 2 * i + 1, ch.rv(Register::from(2 * i + 1))),
                ]))
            }
            let title: Line = Line::from("Registers").bold().blue().centered();
            Table::new(rows, widths).block(Block::bordered().title(title))
        }

        fn timers_table<'a>(d: &Debugger) -> Table<'a> {
            let widths = [Constraint::Fill(1), Constraint::Fill(1)];
            let mut rows: Vec<Row> = vec![];
            let ch = &d.peek();
            rows.push(Row::new([
                format!("sound timer: {}", ch.sound),
                format!("delay timer: {}", ch.delay),
            ]));
            let title: Line = Line::from("Timers").bold().blue().centered();
            Table::new(rows, widths).block(Block::bordered().title(title))
        }

        fn stack<'a>(d: &Debugger) -> Paragraph<'a> {
            let title: Line = Line::from("Stack").bold().blue().centered();
            let ch = &d.peek();
            let sstack = &ch.stack[..ch.sp as usize];
            let text: String = format!("top ---> {:?}", sstack);
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered()
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
                    let raw : RawInstr = RawInstr::from_bytes([c.memory[ix], c.memory[ix + 1]]);
                    format!(
                        "{} {} {}",
                        raw,
                        raw.clone().into_instr(),
                        if i == pc {
                            format!(" <--- pc = {:#06X}", pc)
                        } else {
                            String::from("")
                        }
                    )
                })
            }
            List::new(m).block(Block::bordered().title(title))
        }

        fn help<'a>() -> Paragraph<'a> {
            let title: Line = Line::from("Help").bold().blue().centered();
            let lines = vec![
                Line::from("Chip-8 debugger key bindings:"),
                Line::from(vec!["n".bold(), " step forward".into()]),
                Line::from(vec!["p".bold(), " step backward".into()]),
                Line::from(vec!["d".bold(), " toggle diff".into()]),
                Line::from(vec!["q".bold(), " quit".into()]),
            ];
            let text = Text::from(lines);
            // let text: &str = "Chip-8 debugger key bindings!\n\n\
            // Press `n` and `p` to step forward and backward\n\n
            // Press `Esc`, `Ctrl-C` or `q` to stop running.";
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .alignment(Alignment::Left)
        }

        let root_layout =
            Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)]);
        let [display_area, tools_area] = root_layout.areas(area);
        let [help_area, memory_area, registers_area] = Layout::horizontal([
            Constraint::Percentage(100),
            Default::default(),
            Default::default(),
        ])
        .areas(tools_area);
        let [v_area, stack_area, timers_area] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
        ])
        .areas(registers_area);

        let p1 = display(&self.debugger);
        let mem = memory(&self.debugger);
        let help = help();
        p1.render(display_area, buf);
        Widget::render(mem, memory_area, buf);
        help.render(help_area, buf);
        Widget::render(v_table(&self.debugger), v_area, buf);
        Widget::render(timers_table(&self.debugger), timers_area, buf);
        Widget::render(stack(&self.debugger), stack_area, buf);
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
                command::Command::Redraw => (),
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

pub mod command {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    pub enum Command {
        /// The debugger moves one step forward
        StepForward,
        /// The debugger moves one step backward
        StepBackward,
        /// Exits the application
        Exit,
        /// Redraws the interface
        Redraw,
    }

    impl Command {
        pub fn command_from_event(e: Event) -> Option<Command> {
            match e {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    Self::command_from_key_pressed(key)
                }
                Event::Resize { .. } => Some(Command::Redraw),

                _ => None,
            }
        }

        pub fn command_from_key_pressed(k: KeyEvent) -> Option<Command> {
            match (k.modifiers, k.code) {
                (_, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                    Some(Command::Exit)
                }
                (_, KeyCode::Enter | KeyCode::Char('n') | KeyCode::Right | KeyCode::Char(' ')) => {
                    Some(Command::StepForward)
                }
                (_, KeyCode::Backspace | KeyCode::Char('p') | KeyCode::Left) => {
                    Some(Command::StepBackward)
                }
                _ => None,
            }
        }
    }
}
