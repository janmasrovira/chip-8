use std::*;

pub const SPRITE_BYTES: usize = 5;

type CharSprite = [u8; SPRITE_BYTES];

pub const ZERO: CharSprite = [0xf0, 0x90, 0x90, 0x90, 0xf0];
pub const ONE: CharSprite = [0x20, 0x60, 0x20, 0x20, 0x70];
pub const TWO: CharSprite = [0xf0, 0x10, 0xf0, 0x80, 0xf0];
pub const THREE: CharSprite = [0xf0, 0x10, 0xf0, 0x10, 0xf0];
pub const FOUR: CharSprite = [0x90, 0x90, 0xf0, 0x10, 0x10];
pub const FIVE: CharSprite = [0xf0, 0x80, 0xf0, 0x10, 0xf0];
pub const SIX: CharSprite = [0xf0, 0x80, 0xf0, 0x90, 0xf0];
pub const SEVEN: CharSprite = [0xf0, 0x10, 0x20, 0x40, 0x40];
pub const EIGHT: CharSprite = [0xf0, 0x90, 0xf0, 0x90, 0xf0];
pub const NINE: CharSprite = [0xf0, 0x90, 0xf0, 0x10, 0xf0];
pub const A: CharSprite = [0xf0, 0x90, 0xf0, 0x90, 0x90];
pub const B: CharSprite = [0xe0, 0x90, 0xe0, 0x90, 0xe0];
pub const C: CharSprite = [0xf0, 0x80, 0x80, 0x80, 0x90];
pub const D: CharSprite = [0xe0, 0x90, 0x90, 0x90, 0xe0];
pub const E: CharSprite = [0xf0, 0x80, 0xf0, 0x80, 0xf0];
pub const F: CharSprite = [0xf0, 0x80, 0xf0, 0x80, 0x80];

pub const ALL_CHARS: [&CharSprite; 16] = [
    &ZERO, &ONE, &TWO, &THREE, &FOUR, &FIVE, &SIX, &SEVEN, &EIGHT, &NINE, &A, &B, &C, &D, &E, &F,
];

// Total bytes = 16 * 5 = 80
pub const ALL_CHARS_BYTES: usize = ALL_CHARS.len() * SPRITE_BYTES;

/// Copies all character bytes sequentially starting from the beginning of the array
/// The array needs to have at least length 80
pub fn copy_chars<const N: usize, const START: usize>(m: &mut [u8; N]) {
    assert!(N - START >= ALL_CHARS_BYTES);
    let chunks_x_sprites = m[START..].chunks_mut(SPRITE_BYTES).zip(ALL_CHARS);
    for (chunk, sprite) in chunks_x_sprites {
        chunk.copy_from_slice(sprite);
    }
}
