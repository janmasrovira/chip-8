use super::architecture::*;
use super::base::*;
use super::debugger::*;
use super::language::*;
use bitvec::prelude::*;
use std::fs::*;
use std::io::*;
use std::num::*;
use std::path::PathBuf;
use std::{thread, time};

impl Screen {
    /// XOr bit at the specified position, returns true if the bit switches from
    /// 1 to 0
    pub fn draw_bit(&mut self, row: u16, col: u16, b: bool) -> bool {
        let mrow = row as usize % Self::NROWS;
        let mcol = col as usize % Self::NCOLS;
        let old = self.rows[mrow][mcol];
        let new = old ^ b;
        self.rows[mrow].set(mcol, new);
        old && !new
    }

    pub fn to_string(&self) -> String {
        let mut s: String = String::new();
        for ln in self.rows {
            for c in ln {
                s.push(if c { '█' } else { '.' })
            }
            s.push('\n');
        }
        s
    }

    pub fn print(&self) {
        print!("{}", self.to_string());
    }
}

impl Chip8 {
    pub fn run(&mut self) {
        loop {
            self.run_instr();
            thread::sleep(time::Duration::from_millis(1000 / 100));
        }
    }

    pub fn v(&mut self, r: Register) -> &mut Wrapping<u8> {
        &mut self.registers[r.as_usize()]
    }

    pub fn rv(&self, r: Register) -> u8 {
        self.registers[r.as_usize()].0
    }

    pub fn read_instr(&self) -> Instr {
        let upc = self.pc as usize;
        let bytes: [u8; 2] = self.memory[upc..=upc + 1]
            .try_into()
            .expect("invalid memory access");
        let r: RawInstr = RawInstr::from_bytes(bytes);
        r.into_instr()
    }

    pub fn pc_incr(&mut self) {
        self.pc += 2;
    }

    pub fn pop_stack(&mut self) -> u16 {
        let s = self.stack[self.sp as usize - 1];
        self.sp -= 1;
        s
    }

    pub fn push_stack(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn run_instr(&mut self) {
        let i = self.read_instr();
        match i {
            Instr::System { addr: _ } => {
                self.pc_incr();
            }
            Instr::Clear => {
                self.screen = Screen::new();
                self.pc_incr();
            }
            Instr::Ret => {
                self.pc = self.pop_stack();
                self.pc_incr();
            }
            Instr::Goto { addr: a } => self.pc = a.into(),
            Instr::Call { addr: a } => {
                self.push_stack(self.pc);
                self.pc = a.into();
            }
            Instr::SkipEq { r, c } => {
                if self.rv(r) == c {
                    self.pc_incr();
                }
                self.pc_incr();
            }
            Instr::SkipNEq { r, c } => {
                if self.rv(r) != c {
                    self.pc_incr();
                }
                self.pc_incr();
            }
            Instr::SkipEqV { r, s } => {
                if self.rv(r) == self.rv(s) {
                    self.pc_incr();
                }
                self.pc_incr();
            }
            Instr::Set { r, a } => {
                *self.v(r) = Wrapping(a as u8);
                self.pc_incr();
            }
            Instr::Incr { r, a } => {
                *self.v(r) += Wrapping(a as u8);
                self.pc_incr();
            }
            Instr::Copy { r, s } => {
                *self.v(r) = *self.v(s);
                self.pc_incr();
            }
            Instr::BitOr { r, s } => {
                *self.v(r) = *self.v(r) | *self.v(s);
                self.pc_incr();
            }
            Instr::BitAnd { r, s } => {
                *self.v(r) = *self.v(r) & *self.v(s);
                self.pc_incr();
            }
            Instr::BitXOr { r, s } => {
                *self.v(r) = *self.v(r) ^ *self.v(s);
                self.pc_incr();
            }
            Instr::Add { r, s } => {
                let (n, overflow) = self.rv(r).overflowing_add(self.rv(s));
                *self.v(r) = Wrapping(n);
                *self.v(Register::VF) = Wrapping(overflow as u8);
                self.pc_incr();
            }
            Instr::ShiftR { r } => {
                let (n, overflow) = self.rv(r).overflowing_shr(1);
                *self.v(Register::VF) = Wrapping(overflow as u8);
                *self.v(r) = Wrapping(n);
                self.pc_incr();
            }
            Instr::Sub { r, s } => {
                let (n, borrow) = self.rv(r).overflowing_sub(self.rv(s));
                *self.v(r) = Wrapping(n);
                *self.v(Register::VF) = Wrapping(!borrow as u8);
                self.pc_incr();
            }
            Instr::Lt { r, s } => {
                let (n, borrow) = self.rv(s).overflowing_sub(self.rv(r));
                *self.v(r) = Wrapping(n);
                *self.v(Register::VF) = Wrapping(!borrow as u8);
                self.pc_incr();
            }
            Instr::ShiftL { r } => {
                let (n, overflow) = self.rv(r).overflowing_shl(1);
                *self.v(Register::VF) = Wrapping(overflow as u8);
                *self.v(r) = Wrapping(n);
                self.pc_incr();
            }
            Instr::SkipNEqV { r, s } => {
                if self.rv(r) != self.rv(s) {
                    self.pc_incr();
                }
                self.pc_incr();
            }
            Instr::SetI { n } => {
                self.i = n.into();
                self.pc_incr();
            }
            Instr::Jump { n } => {
                self.pc = self.rv(Register::V0) as u16 + u16::from(n);
            }
            Instr::Rand { r, n } => {
                *self.v(r) = Wrapping((n & rand::random::<u8>()) as u8);
                self.pc_incr();
            }
            Instr::Draw { x, y, height } => {
                let reg_i: usize = self.i as usize;
                let i0 = self.rv(y) as u16;
                let j0 = self.rv(x) as u16;
                let sprite: &[u8] = &self.memory[reg_i..reg_i + height as usize];
                let mut collision: bool = false;
                for (i, line) in sprite.iter().enumerate() {
                    let line_bits: &BitSlice<u8, Msb0> = line.view_bits();
                    for j in 0..8 {
                        collision |=
                            self.screen
                                .draw_bit(i0 + i as u16, j0 + j as u16, line_bits[j]);
                    }
                }
                *self.v(Register::VF) = Wrapping(collision as u8);
                self.pc_incr();
            }
            Instr::Pressed { r } => {
                todo!()
            }
            Instr::NotPressed { r } => {
                todo!()
            }
            Instr::GetDelay { r } => {
                todo!()
            }
            Instr::LoadKey { r } => {
                todo!()
            }
            Instr::SetDelayTimer { r } => {
                todo!()
            }
            Instr::SetSoundTimer { r } => {
                todo!()
            }
            Instr::IncrI { r } => {
                self.i += self.rv(r) as u16;
                self.pc_incr();
            }
            Instr::SpriteAddr { r } => {
                todo!()
            }
            Instr::StoreBCD { r } => {
                let mut v: u16 = self.rv(r) as u16;
                let d1: u8 = (v % 10) as u8;
                v /= 10;
                let d10: u8 = (v % 10) as u8;
                v /= 10;
                let d100: u8 = (v % 10) as u8;
                let i = self.i as usize;
                self.memory[i] = d100;
                self.memory[i + 1] = d10;
                self.memory[i + 2] = d1;
                self.pc_incr();
            }
            Instr::RegDump { x } => {
                let Nibble(n) = x;
                for i in 0..=n as usize {
                    self.memory[i] = self.rv(Register::from(i as u8));
                }
                self.pc_incr();
            }
            Instr::RegLoad { x } => {
                let Nibble(n) = x;
                for i in 0..=n as usize {
                    *self.v(Register::from(i as u8)) = Wrapping(self.memory[i]);
                }
                self.pc_incr();
            }
            Instr::Data(_) => {
                panic!("Data cannot be interpreted as an instruction")
            }
        }
    }

    pub fn load_memory(&mut self, filepath: &PathBuf) -> Result<()> {
        let mut v: Vec<u8> = Vec::new();
        let mut f: File = File::open(filepath)?;
        let len: usize = Read::read_to_end(&mut f, &mut v)?;
        if len >= Chip8::MEM_SIZE - Chip8::CODE_START {
            panic!(
                "The given file size exceeds Chip8 memory.\nFile bytes = {len}; Max bytes = {:?}",
                Chip8::MEM_SIZE
            )
        }
        self.memory[Chip8::CODE_START..Chip8::CODE_START + len].copy_from_slice(&v[..len]);
        Ok(())
    }
}

impl Debugger {
    pub fn new(chip: Chip8) -> Debugger {
        Debugger {
            history: vec![chip],
            p: 0,
        }
    }

    pub fn peek(&self) -> &Chip8 {
        &self.history[self.p]
    }

    pub fn step_back(&mut self) -> bool {
        let possible = self.p > 0;
        if possible {
            self.p -= 1;
        }
        possible
    }

    pub fn step_forward(&mut self) {
        if self.p == self.history.len() - 1 {
            let mut next = self.history.last().unwrap().clone();
            next.run_instr();
            self.history.push(next);
        }
        self.p += 1;
    }
}
