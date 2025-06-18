use super::architecture::*;

pub struct Debugger {
    pub history: Vec<Chip8>,
    pub p: usize,
}
