use super::architecture::*;
use super::language::*;
use std::fs::*;
use std::io::*;
use std::path::*;

pub fn parse_file(filepath: &PathBuf) -> Result<Program> {
    let mut v: Vec<u8> = Vec::new();
    let mut f: File = File::open(filepath)?;
    let _ = Read::read_to_end(&mut f, &mut v);
    Ok(parse_bytes(v))
}

pub fn parse_bytes(input: Vec<u8>) -> Program {
    let s = split(input);
    Program {
        instructions: s.into_iter().map(|i| i.into_instr()).collect(),
    }
}

fn split(input: Vec<u8>) -> Vec<RawInstr> {
    let (chunks, rem): (&[[u8; 2]], &[u8]) = input.as_chunks();
    assert!(
        (*rem).is_empty(),
        "The number of bytes must be a multiple of 2. Leftover bytes = {}",
        (*rem).len()
    );
    chunks.iter().map(|bb| RawInstr::from_bytes(*bb)).collect()
}
