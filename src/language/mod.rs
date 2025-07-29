use super::architecture::*;
use super::base::*;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(PartialEq, Eq, Debug)]
pub struct Address(u16);

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "@{:#05X}", self.0)
    }
}

impl From<Address> for u16 {
    fn from(value: Address) -> u16 {
        let Address(v) = value;
        v
    }
}

impl From<[UNibble; 3]> for Address {
    fn from(value: [UNibble; 3]) -> Self {
        Address(mk_un(value.as_slice()) as u16)
    }
}

/// A raw instruction is a sequence of 4 bytes
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct RawInstr {
    pub nibbles: [Nibble; 4],
}

impl Display for RawInstr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{:#06X}",
            mk_un(self.nibbles.clone().map(|Nibble(u)| u).to_vec().as_slice())
        )
    }
}

impl RawInstr {
    pub fn from_bytes(bytes: [u8; 2]) -> RawInstr {
        let [a, b] = Nibble::byte_to_nibbles(bytes[0]);
        let [c, d] = Nibble::byte_to_nibbles(bytes[1]);
        RawInstr {
            nibbles: [a, b, c, d],
        }
    }

    #[allow(clippy::uninlined_format_args)]
    pub fn into_instr(self) -> Instr {
        fn mk_u8(b: &[UNibble; 2]) -> u8 {
            mk_un((*b).as_slice()) as u8
        }

        let b: [UNibble; 4] = self.nibbles.map(|Nibble(x)| x);
        match b {
            [0, 0, 0xE, 0] => Instr::Clear,
            [0, 0, 0xE, 0xE] => Instr::Ret,
            [0, b @ ..] => Instr::System { addr: b.into() },
            [1, b @ ..] => Instr::Goto { addr: b.into() },
            [2, b @ ..] => Instr::Call { addr: b.into() },
            [3, x, k @ ..] => Instr::SkipEq {
                r: Register::from(x),
                c: mk_u8(&k),
            },
            [4, x, k @ ..] => Instr::SkipNEq {
                r: Register::from(x),
                c: mk_u8(&k),
            },
            [5, x, y, 0] => Instr::SkipEqV {
                r: Register::from(x),
                s: Register::from(y),
            },
            [6, x, k @ ..] => Instr::Set {
                r: Register::from(x),
                a: mk_u8(&k),
            },
            [7, x, k @ ..] => Instr::Incr {
                r: Register::from(x),
                a: mk_u8(&k),
            },
            [8, x, y, 0] => Instr::Copy {
                r: Register::from(x),
                s: Register::from(y),
            },
            [8, x, y, 1] => Instr::BitOr {
                r: Register::from(x),
                s: Register::from(y),
            },
            [8, x, y, 2] => Instr::BitAnd {
                r: Register::from(x),
                s: Register::from(y),
            },
            [8, x, y, 3] => Instr::BitXOr {
                r: Register::from(x),
                s: Register::from(y),
            },
            [8, x, y, 4] => Instr::Add {
                r: Register::from(x),
                s: Register::from(y),
            },
            [8, x, y, 5] => Instr::Sub {
                r: Register::from(x),
                s: Register::from(y),
            },
            [8, x, _, 6] => Instr::ShiftR {
                r: Register::from(x),
            },
            [8, x, y, 7] => Instr::Lt {
                r: Register::from(x),
                s: Register::from(y),
            },
            [8, x, _, 0xE] => Instr::ShiftL {
                r: Register::from(x),
            },
            [9, x, y, 0] => Instr::SkipNEqV {
                r: Register::from(x),
                s: Register::from(y),
            },
            [0xA, n @ ..] => Instr::SetI { n: n.into() },
            [0xB, n @ ..] => Instr::Jump { n: n.into() },
            [0xC, x, k @ ..] => Instr::Rand {
                r: Register::from(x),
                n: mk_u8(&k),
            },
            [0xD, x, y, n] => Instr::Draw {
                x: Register::from(x),
                y: Register::from(y),
                height: n,
            },
            [0xE, x, 9, 0xE] => Instr::Pressed {
                r: Register::from(x),
            },
            [0xE, x, 0xA, 1] => Instr::NotPressed {
                r: Register::from(x),
            },
            [0xF, x, 0, 7] => Instr::GetDelay {
                r: Register::from(x),
            },
            [0xF, x, 0, 0xA] => Instr::LoadKey {
                r: Register::from(x),
            },
            [0xF, x, 1, 5] => Instr::SetDelayTimer {
                r: Register::from(x),
            },
            [0xF, x, 1, 8] => Instr::SetSoundTimer {
                r: Register::from(x),
            },
            [0xF, x, 1, 0xE] => Instr::IncrI {
                r: Register::from(x),
            },
            [0xF, x, 2, 9] => Instr::SpriteAddr {
                r: Register::from(x),
            },
            [0xF, x, 3, 3] => Instr::StoreBCD {
                r: Register::from(x),
            },
            [0xF, x, 5, 5] => Instr::RegDump { x: Nibble::from(x) },
            [0xF, x, 6, 5] => Instr::RegLoad { x: Nibble::from(x) },
            _ => Instr::Data(b),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Instr {
    /// Calls machine code routine. Obsolete instruction that is currently
    /// ignored.
    System {
        addr: Address,
    },

    /// Clears the screen
    Clear,

    /// Returns from a subroutine
    Ret,

    /// Jumps to address
    Goto {
        addr: Address,
    },

    /// Calls subroutine
    Call {
        addr: Address,
    },

    /// Skips the next instruction if r == c
    SkipEq {
        r: Register,
        c: u8,
    },

    /// Skips the next instruction if r != c
    SkipNEq {
        r: Register,
        c: u8,
    },

    /// Skips the next instruction if r = s
    SkipEqV {
        r: Register,
        s: Register,
    },

    /// r := a
    Set {
        r: Register,
        a: u8,
    },

    /// r := r + a
    Incr {
        r: Register,
        a: u8,
    },

    /// r := s
    Copy {
        r: Register,
        s: Register,
    },

    /// r := r || s
    BitOr {
        r: Register,
        s: Register,
    },

    /// r := r && s
    BitAnd {
        r: Register,
        s: Register,
    },

    /// r := r xor s
    BitXOr {
        r: Register,
        s: Register,
    },

    /// r := r + s; if overflow then VF := 1
    Add {
        r: Register,
        s: Register,
    },

    /// least significant bit of r in VF; then r := r >> 1
    ShiftR {
        r: Register,
    },

    /// r := r - s
    Sub {
        r: Register,
        s: Register,
    },

    /// r := s - r; VF := r < s
    Lt {
        r: Register,
        s: Register,
    },

    /// most significant bit of r in VF; then r := r << 1;
    ShiftL {
        r: Register,
    },

    /// Skips the next instruction if r != s
    SkipNEqV {
        r: Register,
        s: Register,
    },

    /// I := n
    SetI {
        n: U12,
    },

    /// PC := V0 + n
    Jump {
        n: U12,
    },

    /// r := rand() % n
    Rand {
        r: Register,
        n: u8,
    },

    /// Draws a sprite at position (x, y). VF is set to 1 if there is a collision
    Draw {
        x: Register,
        y: Register,
        height: u8,
    },

    /// Skips next instruction if key.pressed = r
    Pressed {
        r: Register,
    },

    /// Skips next instruction if key.pressed != r
    NotPressed {
        r: Register,
    },

    /// r := get_delay
    GetDelay {
        r: Register,
    },

    /// r := get_key
    LoadKey {
        r: Register,
    },

    /// r := delay_timer
    SetDelayTimer {
        r: Register,
    },

    /// r := sound_timer
    SetSoundTimer {
        r: Register,
    },

    /// I := I + r
    IncrI {
        r: Register,
    },

    /// I := sprite_addr r
    SpriteAddr {
        r: Register,
    },

    StoreBCD {
        r: Register,
    },

    /// mem[I] = V0, .., mem[I + x] = Vx
    RegDump {
        x: Nibble,
    },

    /// V0 = I, .., Vx = I + x
    RegLoad {
        x: Nibble,
    },

    /// data
    Data([UNibble; 4]),
}

impl Display for Instr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Instr::System { addr } => write!(f, "SYS {addr}"),
            Instr::Clear => write!(f, "CLS"),
            Instr::Ret => write!(f, "RET"),
            Instr::Goto { addr } => write!(f, "JP {addr}"),
            Instr::Call { addr } => write!(f, "CALL {addr}"),
            Instr::SkipEq { r, c } => write!(f, "SE {r}, {c}"),
            Instr::SkipNEq { r, c } => write!(f, "SNE {r}, {c}"),
            Instr::SkipEqV { r, s } => write!(f, "SE {r}, {s}"),
            Instr::Set { r, a } => write!(f, "LD {r}, {a}"),
            Instr::Incr { r, a } => write!(f, "ADD {r}, {a}"),
            Instr::Copy { r, s } => write!(f, "LD {r}, {s}"),
            Instr::BitOr { r, s } => write!(f, "OR {r}, {s}"),
            Instr::BitAnd { r, s } => write!(f, "AND {r}, {s}"),
            Instr::BitXOr { r, s } => write!(f, "XOR {r}, {s}"),
            Instr::Add { r, s } => write!(f, "ADD {r}, {s}"),
            Instr::Sub { r, s } => write!(f, "SUB {r}, {s}"),
            Instr::ShiftR { r } => write!(f, "SHR {r}"),
            Instr::Lt { r, s } => write!(f, "SUBN {r}, {s}"),
            Instr::ShiftL { r } => write!(f, "SHL {r}"),
            Instr::SkipNEqV { r, s } => write!(f, "SNE {r}, {s}"),
            Instr::SetI { n } => write!(f, "LD I, {n}"),
            Instr::Jump { n } => write!(f, "JP V0, {n}"),
            Instr::Rand { r, n } => write!(f, "RND {r}, {n}"),
            Instr::Draw { x, y, height } => write!(f, "DRW {x}, {y}, {height}"),
            Instr::Pressed { r } => write!(f, "SKP {r}"),
            Instr::NotPressed { r } => write!(f, "SKPN {r}"),
            Instr::GetDelay { r } => write!(f, "LD {r}, DT"),
            Instr::LoadKey { r } => write!(f, "LD {r}, K"),
            Instr::SetDelayTimer { r } => write!(f, "LD DT, {r}"),
            Instr::SetSoundTimer { r } => write!(f, "LD ST, {r}"),
            Instr::IncrI { r } => write!(f, "ADD I, {r}"),
            Instr::SpriteAddr { r } => write!(f, "LD F, {r}"),
            Instr::StoreBCD { r } => write!(f, "LD B, {r}"),
            Instr::RegDump { x } => write!(f, "LD [I], {x}"),
            Instr::RegLoad { x } => write!(f, "LD {x}, [I]"),
            Instr::Data(_) => write!(f, "DATA"),
        }
    }
}
