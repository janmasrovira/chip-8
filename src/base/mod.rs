/// unchecked nibble
pub type UNibble = u8;

/// checked nibble, 0 <= nibble <= 15
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Nibble(pub UNibble);

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

impl From<Nibble> for u8 {
    fn from(value: Nibble) -> u8 {
        let Nibble(x) = value;
        x
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct U12(u16);

impl From<U12> for u16 {
    fn from(value: U12) -> u16 {
        let U12(v) = value;
        v
    }
}

impl From<[UNibble; 3]> for U12 {
    fn from(value: [u8; 3]) -> Self {
        U12(mk_un(value.as_slice()) as u16)
    }
}

pub fn mk_un(bs: &[UNibble]) -> u32 {
    let mut ret: u32 = 0;
    for (i, b) in bs.iter().rev().enumerate() {
        ret += (*b as u32) * (16u32.pow(i as u32));
    }
    ret
}
