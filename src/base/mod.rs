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

#[derive(PartialEq, Eq, Debug)]
pub struct U12([Nibble; 3]);

impl From<[UNibble; 3]> for U12 {
    fn from(value: [u8; 3]) -> Self {
        U12(value.map(Nibble::new))
    }
}
