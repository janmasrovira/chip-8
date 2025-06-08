/// unchecked nibble
type UNibble = u8;

/// checked nibble, 0 <= nibble <= 15
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Nibble(UNibble);

impl Nibble {
    pub fn check(num: u8) {
        assert!(
            num <= 15,
            "nibble must satisfy 0 <= nibble <= 15. Actual value = {num}"
        );
    }

    pub fn new(nibble: u8) -> Self {
        Self::check(nibble);
        Nibble(nibble)
    }

    pub fn byte_to_nibbles(b: u8) -> [Nibble; 2] {
        [Nibble(b / 16), Nibble(b % 16)]
    }
}

impl From<u8> for Nibble {
    fn from(value: u8) -> Self {
        Nibble::new(value)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Address([Nibble; 3]);

impl From<[UNibble; 3]> for Address {
    fn from(value: [UNibble; 3]) -> Self {
        Address(value.map(Nibble::new))
    }
}
#[derive(PartialEq, Eq, Debug)]
pub struct U12([Nibble; 3]);

impl From<[UNibble; 3]> for U12 {
    fn from(value: [u8; 3]) -> Self {
        U12(value.map(Nibble::new))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Program {
    pub instructions: Vec<Instr>,
}

/// The state of a Chip8
#[derive(PartialEq, Eq, Debug)]
pub struct Chip8 {
    i: U12,
    pc: u16,
    registers: [u16; 16],
}

/// A raw instruction is a sequence of 4 bytes
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct RawInstr {
    pub nibbles: [Nibble; 4],
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
        println!("instr = {:?}", self);
        fn mk_un(bs: &[UNibble]) -> u32 {
            let mut ret: u32 = 0;
            let i: u32 = 0;
            for b in bs.iter().rev() {
                ret += (*b as u32) * ((4 * 2) ^ i);
            }
            ret
        }

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
                r: Register::from(x),
                s: Register::from(y),
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
            x => Instr::Other(b),
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

    /// TODO
    Draw {
        r: Register,
        s: Register,
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

    /// I = V0, .., I + x = Vx
    RegDump {
        x: Nibble,
    },

    /// V0 = I, .., Vx = I + x
    RegLoad {
        x: Nibble,
    },

    /// TODO used to store data?
    Other([UNibble; 4]),
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

impl Register {}
