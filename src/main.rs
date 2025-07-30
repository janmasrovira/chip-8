#![feature(slice_as_array)]
mod architecture;
mod base;
mod cli;
mod debugger;
mod emulator;
mod font;
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
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        fn v_table<'a>(d: &Debugger) -> Table<'a> {
            let widths = [Constraint::Fill(1), Constraint::Fill(1)];
            let mut rows: Vec<Row> = vec![];
            let ch = &d.peek();
            let pch = &d.peek_prev();
            let pp_helper = |reg_name: String, before: Option<u16>, now: u16| -> Line {
                if d.diff
                    && let Some(prev) = before
                    && (prev != now)
                {
                    Line::from(vec![
                        Span::from(format!("{reg_name}: ")),
                        Span::from(prev.to_string()).red(),
                        Span::from(" → "),
                        Span::from(now.to_string()).green(),
                    ])
                } else {
                    Line::from(format!("{reg_name}: {now}"))
                }
            };
            let pp_register = |r: Register| -> Line {
                pp_helper(r.to_string(), pch.map(|c| c.rv(r).into()), ch.rv(r).into())
            };
            rows.push(Row::new([pp_helper("I".into(), pch.map(|c| c.i), ch.i)]));
            for i in 0..8 {
                rows.push(Row::new([
                    pp_register(Register::from(2 * i)),
                    pp_register(Register::from(2 * i + 1)),
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
            let string = format!("Memory (step {}/{})", d.step_number(), d.step_max());
            let title: Line = Line::from(string).bold().blue().centered();
            let c = &d.peek();
            let pc: i32 = c.pc as i32;
            const H: i32 = 15 * 2;
            let mut m: Vec<Span> = vec![];
            for i in (pc - H..=pc + H).step_by(2) {
                m.push(if i < 0 || i + 1 >= Chip8::MEM_SIZE as i32 {
                    "-".into()
                } else {
                    let ix = i as usize;
                    let raw: RawInstr = RawInstr::from_bytes([c.memory[ix], c.memory[ix + 1]]);
                    let s = Span::from(format!(
                        "{raw} {} {}",
                        raw.clone().into_instr(),
                        if i == pc {
                            format!(" <--- pc = {:#06X}", pc)
                        } else {
                            String::from("")
                        }
                    ));
                    if i == pc { s.bold() } else { s }
                })
            }
            List::new(m).block(Block::bordered().title(title))
        }

        fn help<'a>() -> Paragraph<'a> {
            let title: Line = Line::from("Help").bold().blue().centered();
            let lines = vec![
                Line::from("Chip-8 debugger key bindings:"),
                Line::from(vec!["n".bold(), " step forward".into()]),
                Line::from(vec!["N".bold(), " 10 steps forward".into()]),
                Line::from(vec!["p".bold(), " step backward".into()]),
                Line::from(vec!["P".bold(), " 10 steps backward".into()]),
                Line::from(vec!["d".bold(), " toggle diff".into()]),
                Line::from(vec!["q".bold(), " quit".into()]),
            ];
            let text = Text::from(lines);
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
                command::Command::ToggleDiff => self.debugger.diff = !self.debugger.diff,
                command::Command::StepForward => self.debugger.step_forward(),
                command::Command::BigStepForward => self.debugger.steps_forward(10),
                command::Command::BigStepBackward => self.debugger.steps_back(10),
                command::Command::StepBackward => {
                    let _ = self.debugger.step_back();
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
        /// The debugger moves 10 steps forward
        BigStepForward,
        /// The debugger moves one step backward
        StepBackward,
        /// The debugger moves 10 steps backward
        BigStepBackward,
        /// Exits the application
        Exit,
        /// Redraws the interface
        Redraw,
        /// Toggles the debugger's visual diff
        ToggleDiff,
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
                (_, KeyCode::Char('N')) => Some(Command::BigStepForward),
                (_, KeyCode::Char('P')) => Some(Command::BigStepBackward),
                (_, KeyCode::Char('d')) => Some(Command::ToggleDiff),
                (_, KeyCode::Backspace | KeyCode::Char('p') | KeyCode::Left) => {
                    Some(Command::StepBackward)
                }
                _ => None,
            }
        }
    }
}
