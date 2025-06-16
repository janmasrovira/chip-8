use super::architecture::*;

pub struct Debugger {
    pub history: Vec<Chip8>,
    pub p: usize,
}

pub mod command {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    pub enum Command {
        StepForward,
        StepBackward,
        Exit,
    }

    impl Command {
        pub fn command_from_event(e: Event) -> Option<Command> {
            match e {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    Self::command_from_key_pressed(key)
                }
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
