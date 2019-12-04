use std::cell::RefCell;
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::process::exit;

struct Memory {
    bucket: RefCell<Vec<usize>>,
}

impl Memory {
    fn read(&self, index: usize) -> usize {
        self.bucket.borrow()[index]
    }

    fn write(&self, index: usize, value: usize) {
        self.bucket.borrow_mut()[index] = value;
    }
}

#[derive(PartialEq, Eq, Debug)]
enum OpCode {
    Add,
    Mul,
    Halt,
    Unknown,
}

impl From<usize> for OpCode {
    fn from(i: usize) -> OpCode {
        match i {
            1 => OpCode::Add,
            2 => OpCode::Mul,
            99 => OpCode::Halt,
            _ => OpCode::Unknown,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Instruction(OpCode, usize, usize, usize);

struct IntCode<'ic> {
    mem: &'ic mut Memory,
    ic: usize,
    width: usize,
}

impl Iterator for IntCode<'_> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.ic * self.width;
        self.ic = self.ic + 1;

        match OpCode::from(self.mem.read(offset)) {
            OpCode::Halt => None,
            OpCode::Unknown => {
                println!(
                    "Invalid OpCode: {} at position {}",
                    self.mem.read(self.ic),
                    self.ic * self.width
                );
                exit(1);
            }
            code => Some(Instruction(
                code,
                self.mem.read(offset + 1),
                self.mem.read(offset + 2),
                self.mem.read(offset + 3),
            )),
        }
    }
}

impl IntCode<'_> {
    fn run_program(&mut self) {
        while let Some(i) = self.next() {
            self.execute(i);
        }
    }

    fn execute(&self, i: Instruction) {
        match i.0 {
            OpCode::Add => self.add(i.1, i.2, i.3),
            OpCode::Mul => self.mul(i.1, i.2, i.3),
            _ => (),
        }
    }

    fn add(&self, op1: usize, op2: usize, op3: usize) {
        self.mem.write(op3, self.mem.read(op1) + self.mem.read(op2))
    }

    fn mul(&self, op1: usize, op2: usize, op3: usize) {
        self.mem.write(op3, self.mem.read(op1) * self.mem.read(op2))
    }
}

fn main() {
    let mut source = File::open(Path::new(&args().next_back().unwrap())).unwrap();

    for x in 0..100 {
        for y in 0..100 {
            let mut buf = Vec::<usize>::new();
            load_program(&mut buf, &mut source);

            let mut memory = Memory {
                bucket: RefCell::new(buf),
            };

            let mut intcode = IntCode {
                mem: &mut memory,
                ic: 0,
                width: 4,
            };

            intcode.mem.write(1, x);
            intcode.mem.write(2, y);

            intcode.run_program();

            if intcode.mem.read(0) == 19690720 {
                println!("x: {}, y: {}", x, y);
                exit(0);
            }

            source.seek(SeekFrom::Start(0)).unwrap();
        }
    }

    //let mut memory = Memory {
    //    bucket: RefCell::new(buf),
    //};

    //let mut intcode = IntCode {
    //    mem: &mut memory,
    //    ic: 0,
    //    width: 4,
    //};

    //intcode.run_program();

    //println!("{}", intcode.mem.read(0));
}

fn load_program(mem: &mut Vec<usize>, file: &mut File) {
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    let _ = content
        .as_mut_str()
        .trim_end()
        .split(',')
        .map(|x| {
            match x.parse::<usize>() {
                Ok(n) => mem.push(n),
                Err(_) => println!("Parse failed: {}", x),
            }
            x
        })
        .collect::<Vec<&str>>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_instruction() {
        let mut memory = memory();
        let mut intcode = IntCode {
            mem: &mut memory,
            ic: 0,
            width: 4,
        };

        assert_eq!(intcode.next(), Some(Instruction(OpCode::Add, 10, 11, 12)));
    }

    #[test]
    fn get_multiple_instructions() {
        let mut memory = memory();
        let mut intcode = IntCode {
            mem: &mut memory,
            ic: 0,
            width: 4,
        };

        intcode.next();

        assert_eq!(intcode.next(), Some(Instruction(OpCode::Mul, 12, 10, 12)));
    }

    #[test]
    fn execution() {
        let mut memory = memory();
        let mut intcode = IntCode {
            mem: &mut memory,
            ic: 0,
            width: 4,
        };

        let i = intcode.next().unwrap();
        intcode.execute(i);

        assert_eq!(intcode.mem.read(12), 3);
    }

    fn memory() -> Memory {
        Memory {
            bucket: RefCell::new(vec![1, 10, 11, 12, 2, 12, 10, 12, 99, 10, 3, 0, 0]),
        }
    }
}
