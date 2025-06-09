use super::base::*;
use bitvec::prelude::*;

/// The state of a Chip8
#[derive(PartialEq, Eq, Debug)]
pub struct Chip8 {
    i: u16,
    pc: u16,
    sp: u8,
    delay: u8,
    sound: u8,
    stack: [u16; 16],
    registers: [u16; 16],
    display: Display,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Display {
    display: [BitArr!(for 64, in u64); 32]
}

impl Display {
    pub fn new() -> Self {
        Display {
            display: [BitArray::ZERO; 32]
        }

    }
}

/// A data register containing a byte (u8)
#[derive(PartialEq, Eq, Debug)]
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

impl From<Register> for u8 {
    fn from(n: Register) -> u8 {
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
