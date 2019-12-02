use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

enum OpCode {
    Add,
    Mul,
    Halt,
    Unknown,
}

impl From<u8> for OpCode {
    fn from(i: u8) -> OpCode {
        match i {
            1 => OpCode::Add,
            2 => OpCode::Mul,
            99 => OpCode::Halt,
            _ => OpCode::Unknown,
        }
    }
}

struct Instruction(OpCode, u8, u8, u8);

struct IntCode<'ic> {
    mem: &'ic mut Vec<u8>,
    pos: usize,
}

impl Iterator for IntCode<'_> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos <= (self.mem.len() - 4) {
            self.pos = self.pos + 4;

            match OpCode::from(self.mem[self.pos]) {
                OpCode::Halt => None,
                OpCode::Unknown => {
                    println!(
                        "Invalid OpCode: {} at position {}",
                        self.mem[self.pos], self.pos
                    );
                    exit(1)
                }
                code => Some(Instruction(
                    code,
                    self.mem[self.pos + 1],
                    self.mem[self.pos + 2],
                    self.mem[self.pos + 3],
                )),
            }
        } else {
            None
        }
    }
}

impl IntCode<'_> {
    fn execute(&mut self) {
        for i in self {
            match i.0 {
                OpCode::Add => Self::add(self.mem, i.1, i.2, i.3),
                OpCode::Mul => Self::mul(self.mem, i.1, i.2, i.3),
                _ => (),
            }
        }
    }

    fn add(mem: &mut Vec<u8>, op1: u8, op2: u8, op3: u8) {
        mem[op3 as usize] = mem[op1 as usize] + mem[op2 as usize];
    }

    fn mul(mem: &mut Vec<u8>, op1: u8, op2: u8, op3: u8) {
        mem[op3 as usize] = mem[op1 as usize] * mem[op2 as usize];
    }
}

fn main() {
    let source = File::open(Path::new(&args().next_back().unwrap())).unwrap();
    let memory = Vec::<u8>::new();
}

fn load_program(mem: &Vec<u8>, file: String) {}

fn execute(i: Instruction, mem: &Vec<u8>) {}
