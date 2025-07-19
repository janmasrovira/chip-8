use super::base::*;
use bitvec::prelude::*;
use std::num::*;

/// The state of a Chip8
#[derive(Debug, Clone)]
pub struct Chip8 {
    /// memory. Memory space from 0x0 to 0x1FF is unused.
    pub memory: [u8; Chip8::MEM_SIZE],
    /// I register. Only the rightmost 12 digits are usually used
    pub i: u16,
    /// program counter
    pub pc: u16,
    /// stack pointer. Points to the next free position in the stack
    pub sp: u8,
    /// delay register
    pub delay: u8,
    /// sound register
    pub sound: u8,
    /// the stack
    pub stack: [u16; 16],
    /// the Vx registers
    pub registers: [Wrapping<u8>; 16],
    /// the display state
    pub screen: Screen,
}

impl Chip8 {
    // TODO revise. I think the memory should be less
    pub const MEM_SIZE: usize = 4096;

    /// Code starts at memory[CODE_START]
    pub const CODE_START: usize = 0x200;

    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; Self::MEM_SIZE],
            i: 0,
            pc: Self::CODE_START as u16,
            sp: 0,
            delay: 0,
            sound: 0,
            stack: [0; 16],
            registers: [Wrapping(0); 16],
            screen: Screen::new(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Screen {
    /// Screen has 32 lines and 64 columns
    pub rows: [BitArr!(for 64, in u64); 32],
}

impl Screen {
    pub const NROWS: usize = 32;
    pub const NCOLS: usize = 64;

    pub fn new() -> Self {
        Screen {
            rows: [BitArray::ZERO; Self::NROWS],
        }
    }
}

/// A data register
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Register {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}

impl Register {
    pub fn as_usize(&self) -> usize {
        u8::from(self) as usize
    }
}

impl From<UNibble> for Register {
    fn from(n: UNibble) -> Self {
        Register::from(Nibble::new(n))
    }
}

impl From<Nibble> for Register {
    fn from(n: Nibble) -> Self {
        let Nibble(nb) = n;
        match nb {
            0 => Register::V0,
            1 => Register::V1,
            2 => Register::V2,
            3 => Register::V3,
            4 => Register::V4,
            5 => Register::V5,
            6 => Register::V6,
            7 => Register::V7,
            8 => Register::V8,
            9 => Register::V9,
            10 => Register::VA,
            11 => Register::VB,
            12 => Register::VC,
            13 => Register::VD,
            14 => Register::VE,
            15 => Register::VF,
            _ => panic!("impossible"),
        }
    }
}

impl From<&Register> for u8 {
    fn from(n: &Register) -> u8 {
        match n {
            Register::V0 => 0,
            Register::V1 => 1,
            Register::V2 => 2,
            Register::V3 => 3,
            Register::V4 => 4,
            Register::V5 => 5,
            Register::V6 => 6,
            Register::V7 => 7,
            Register::V8 => 8,
            Register::V9 => 9,
            Register::VA => 10,
            Register::VB => 11,
            Register::VC => 12,
            Register::VD => 13,
            Register::VE => 14,
            Register::VF => 15,
        }
    }
}
