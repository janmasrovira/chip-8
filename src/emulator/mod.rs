use super::architecture::*;
use super::language::*;
use bitvec::prelude::*;
use std::fs::*;
use std::io::*;
use std::path::PathBuf;
use std::{thread, time};

impl Display {
    /// XOr bit at the specified position, returns true if the bit switches from
    /// 1 to 0
    pub fn draw_bit(&mut self, x: u16, y: u16, b: bool) -> bool {
        let mi = y as usize % Self::NROWS;
        let mj = x as usize % Self::NCOLS;
        let old = self.display[mi][mj];
        let new = old ^ b;
        self.display[mi].set(mj, new);
        old && !new
    }

    pub fn print(&self) {
        for ln in self.display {
            for c in ln {
                if c { print!("X") } else { print!(".") }
            }
            println!()
        }
    }
}

impl Chip8 {
    pub fn run(&mut self) {
        loop {
            self.run_instr();
            thread::sleep(time::Duration::from_millis(1000 / 100));
        }
    }

    pub fn v(&mut self, r: Register) -> &mut u16 {
        &mut self.registers[r.as_usize()]
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
        let s = self.stack[self.sp as usize];
        self.sp -= 1;
        s
    }

    pub fn push_stack(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn run_instr(&mut self) {
        let i = self.read_instr();
        println!("pc = {}, run {:?}", self.pc, i);
        // self.display.print();
        match i {
            Instr::System { addr: _ } => {
                self.pc_incr();
            }
            Instr::Clear => {
                self.display = Display::new();
                self.pc_incr();
            }
            Instr::Ret => self.pc = self.pop_stack(),
            Instr::Goto { addr: a } => self.pc = a.into(),
            Instr::Call { addr: a } => {
                self.push_stack(self.pc);
                self.pc = a.into();
            }
            Instr::SkipEq { r, c } => {
                if *self.v(r) == c as u16 {
                    self.pc_incr();
                }
                self.pc_incr();
            }
            Instr::SkipNEq { r, c } => {
                if *self.v(r) != c as u16 {
                    self.pc_incr();
                }
                self.pc_incr();
            }
            Instr::SkipEqV { r, s } => {
                if *self.v(r) == *self.v(s) {
                    self.pc_incr();
                }
                self.pc_incr();
            }
            Instr::Set { r, a } => {
                *self.v(r) = a as u16;
                self.pc_incr();
            }
            Instr::Incr { r, a } => {
                *self.v(r) += a as u16;
                self.pc_incr();
            }
            Instr::Copy { r, s } => {
                *self.v(r) = *self.v(s);
                self.pc_incr();
            }
            Instr::BitOr { r, s } => {
                *self.v(r) |= *self.v(s);
                self.pc_incr();
            }
            Instr::BitAnd { r, s } => {
                *self.v(r) &= *self.v(s);
                self.pc_incr();
            }
            Instr::BitXOr { r, s } => {
                *self.v(r) ^= *self.v(s);
                self.pc_incr();
            }
            Instr::Add { r, s } => {
                let (n, overflow) = self.v(r).overflowing_add(*self.v(s));
                *self.v(r) = n;
                *self.v(Register::VF) = overflow as u16;
                self.pc_incr();
            }
            Instr::ShiftR { r } => {
                let (n, overflow) = self.v(r).overflowing_shr(1);
                *self.v(Register::VF) = overflow as u16;
                *self.v(r) = n;
                self.pc_incr();
            }
            Instr::Sub { r, s } => {
                let (n, borrow) = self.v(r).overflowing_sub(*self.v(s));
                *self.v(r) = n;
                *self.v(Register::VF) = !borrow as u16;
                self.pc_incr();
            }
            Instr::Lt { r, s } => {
                let (n, borrow) = self.v(s).overflowing_sub(*self.v(r));
                *self.v(r) = n;
                *self.v(Register::VF) = !borrow as u16;
                self.pc_incr();
            }
            Instr::ShiftL { r } => {
                let (n, overflow) = self.v(r).overflowing_shl(1);
                *self.v(Register::VF) = overflow as u16;
                *self.v(r) = n;
                self.pc_incr();
            }
            Instr::SkipNEqV { r, s } => {
                if *self.v(r) != *self.v(s) {
                    self.pc_incr();
                }
                self.pc_incr();
            }
            Instr::SetI { n } => {
                self.i = n.into();
                self.pc_incr();
            }
            Instr::Jump { n } => {
                self.pc = *self.v(Register::V0) + u16::from(n);
            }
            Instr::Rand { r, n } => {
                *self.v(r) = (n & rand::random::<u8>()) as u16;
                self.pc_incr();
            }
            Instr::Draw { r, s, height } => {
                let reg_i: usize = self.i as usize;
                let i0 = *self.v(r);
                let j0 = *self.v(s);
                let sprite: &[u8] = &self.memory[reg_i..reg_i + height as usize];
                let mut collision: bool = false;
                for (i, line) in sprite.iter().enumerate() {
                    let line_bits: &BitSlice<u8, Msb0> = line.view_bits();
                    for j in 0..8 {
                        collision |=
                            self.display
                                .draw_bit(i0 + i as u16, j0 + j as u16, line_bits[j]);
                    }
                }
                *self.v(Register::VF) = collision as u16;
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
                self.i += *self.v(r);
                self.pc_incr();
            }
            Instr::SpriteAddr { r } => {
                todo!()
            }
            Instr::StoreBCD { r } => {
                todo!()
            }
            Instr::RegDump { x } => {
                todo!()
            }
            Instr::RegLoad { x } => {
                todo!()
            }
            Instr::Data(_) => {
                panic!("Data cannot be executed")
            }
        }
    }

    pub fn load_memory(&mut self, filepath: &PathBuf) -> Result<()> {
        let mut v: Vec<u8> = Vec::new();
        let mut f: File = File::open(filepath)?;
        let len: usize = Read::read_to_end(&mut f, &mut v)?;
        if len >= Chip8::MEM_SIZE {
            panic!(
                "The given file size exceeds Chip8 memory.\nFile bytes = {len}; Max bytes = {:?}",
                Chip8::MEM_SIZE
            )
        }
        self.memory[Chip8::CODE_START..Chip8::CODE_START + len].copy_from_slice(&v[..len]);
        println!("{:?}", self.memory);
        Ok(())
    }
}
