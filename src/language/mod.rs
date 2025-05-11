pub type Address = u16;
pub type U24 = u32;
pub type U12 = u16;

use std::iter;

pub struct Program {
    pub instructions : Vec<Instr>,
}


/// The state of a Chip8
pub struct Chip8 {
    i : U12,
    pc : u16,
    registers : [u16; 16],
}


impl Chip8 {
}

/// A raw instruction is a sequence of 4 bytes
type RawInstr = [u8; 4];

/// A Chip-8 instruction. https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
pub enum Instr {
    /// Calls machine code routine
    Call { addr: Address },

    /// Clears the screen
    Clear,

    /// Returns from a subroutine
    Ret,

    /// Jumps to address
    Goto { addr: Address },

    /// Calls subroutine
    Fun { addr: Address },

    /// Skips the next instruction if r == c
    TestEq { r: Register, c: u8 },

    /// Skips the next instruction if r != c
    TestNEq { r: Register, c: u8 },

    /// Skips the next instruction if r = s
    TestEqV { r: Register, s: Register },

    /// r := a
    Set { x: Register, a: u8 },

    /// r := a
    Incr { x: Register, a: u8 },

    /// r := s
    Copy { r: Register, s: Register },

    /// r := r || s
    BitOr { r: Register, s: Register },

    /// r := r && s
    BitAnd { r: Register, s: Register },

    /// r := r xor s
    BitXOr { r: Register, s: Register },

    /// r := r - s; if overflow then VF := 1
    Add { r: Register, s: Register },

    /// r := r - s
    Sub { r: Register, s: Register },

    /// least significant bit of r in VF; then r := r >> 1
    ShiftR { r: Register },

    /// r := s - r; VF := r ≤ s
    Le { r: Register, s: Register },

    /// most significant bit of r in VF; then r := r << 1;
    ShiftL { r: Register },

    /// Skips the next instruction if r != s
    TestNEqV { r: Register, s: Register },

    /// I := n
    SetI { n: U24 },

    /// PC := V0 + n
    SetPc { n: U24 },

    /// r := rand() % n
    Rand { r: Register, n: u16 },

    /// TODO
    Draw {
        r: Register,
        s: Register,
        height: u8,
    },

    /// Skips next instruction if key.pressed = r
    Pressed { r: Register },

    /// Skips next instruction if key.pressed != r
    NotPressed { r: Register },

    /// r := get_delay
    GetDelay { r: Register },

    /// r := get_key
    GetKey { r: Register },

    /// r := delay_timer
    SetDelayTimer { r: Register },

    /// r := sound_timer
    SetSoundTimer { r: Register },

    /// I := I + r
    IncrI { r: Register },

    /// I := sprite_addr r
    SpriteAddr { r: Register },

    /// TODO
    RegDump {},

    /// TODO
    RegLoad {},
}

/// A data register containing a byte (u8)
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
    fn to_usize(r : Register) -> usize {
       1
    }
}
