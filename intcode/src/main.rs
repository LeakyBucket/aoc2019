use std::cell::RefCell;
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::process::exit;

struct Memory {
    bucket: RefCell<Vec<i32>>,
}

impl Memory {
    fn read(&self, index: usize) -> i32 {
        self.bucket.borrow()[index]
    }

    fn write(&self, index: usize, value: i32) {
        self.bucket.borrow_mut()[index] = value;
    }
}

#[derive(PartialEq, Eq, Debug)]
enum OpCode {
    Add,
    Mul,
    Halt,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    Unknown,
}

impl From<i32> for OpCode {
    fn from(i: i32) -> OpCode {
        match i {
            1 => OpCode::Add,
            2 => OpCode::Mul,
            3 => OpCode::Input,
            4 => OpCode::Output,
            5 => OpCode::JumpIfTrue,
            6 => OpCode::JumpIfFalse,
            7 => OpCode::LessThan,
            8 => OpCode::Equals,
            99 => OpCode::Halt,
            _ => OpCode::Unknown,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ParameterMode {
    Position,
    Immediate,
}

impl From<i32> for ParameterMode {
    fn from(f: i32) -> ParameterMode {
        match f {
            1 => ParameterMode::Immediate,
            _ => ParameterMode::Position,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Instruction {
    op: OpCode,
    pub args: [Option<i32>;3],
    modes: [ParameterMode;3],
    len: usize,
}

impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {
            op: OpCode::Unknown,
            args: [None, None, None],
            modes: [ParameterMode::Position, ParameterMode::Position, ParameterMode::Position],
            len: 0,
        }
    }
}

impl Instruction {
    fn new(label: i32) -> Self {
        let label_parts = Self::process_label(label);
        let opcode = OpCode::from(label_parts.0);
        let modes = [ParameterMode::from(label_parts.1), ParameterMode::from(label_parts.2), ParameterMode::from(label_parts.3)];
        let len = Self::len(&opcode);

        Instruction {
            op: opcode,
            len,
            modes,
            ..Default::default()
        }
    }

    fn process_label(label: i32) -> (i32, i32, i32, i32) {
        let mut label = label;
        let mut parts: [i32;4] = [0;4];

        for pos in (1..5).rev() {
            let div = 10_i32.pow(pos);
            let mode = label/div;

            parts[(pos - 1) as usize] = mode;

            label = label - (mode * div);
        }

        //dbg!(&label);
        //dbg!(&parts);

        (label, parts[1], parts[2], parts[3])
    }

    fn len(op: &OpCode) -> usize {
        match op {
            OpCode::Add => 4,
            OpCode::Mul => 4,
            OpCode::Halt => 1,
            OpCode::Input => 2,
            OpCode::Output => 2,
            _ => 0
        }
    }
}

struct IntCode<'ic> {
    mem: &'ic mut Memory,
    ic: usize,
}

impl Iterator for IntCode<'_> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        let mut instruction = Instruction::new(self.mem.read(self.ic));

        match &instruction.op {
            OpCode::Halt => None,
            OpCode::Unknown => {
                println!(
                    "Invalid OpCode: {} at position {}",
                    self.mem.read(self.ic),
                    self.ic
                );
                exit(1);
            }
            _ => {
                for i in 0..(instruction.len - 1) {
                    instruction.args[i] = Some(self.mem.read(self.ic + (i + 1)));
                }
                self.ic = self.ic + instruction.len;
                Some(instruction)
            },
        }
    }
}

impl IntCode<'_> {
    fn run_program(&mut self, input: Option<i32>) {
        while let Some(i) = self.next() {
            self.execute(i, input);
        }
    }

    fn execute(&self, i: Instruction, input: Option<i32>) {
        match i.op {
            OpCode::Add => self.add(i),
            OpCode::Mul => self.mul(i),
            OpCode::Input => self.mem.write(i.args[0].unwrap() as usize, input.unwrap()),
            OpCode::Output => println!("Output: {}", self.mem.read(i.args[0].unwrap() as usize)),
            _ => (),
        }
    }

    fn add(&self, i: Instruction) {
        let op1 = match i.modes[0] {
            ParameterMode::Immediate => i.args[0].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[0].unwrap() as usize),
        };
        let op2 = match i.modes[1] {
            ParameterMode::Immediate => i.args[1].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[1].unwrap() as usize),
        };

        self.mem.write(i.args[2].unwrap() as usize, op1 + op2);
    }

    fn mul(&self, i: Instruction) {
        let op1 = match i.modes[0] {
            ParameterMode::Immediate => i.args[0].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[0].unwrap() as usize),
        };
        let op2 = match i.modes[1] {
            ParameterMode::Immediate => i.args[1].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[1].unwrap() as usize),
        };

        self.mem.write(i.args[2].unwrap() as usize, op1 * op2);
    }
}

fn main() {
    let mut source = File::open(Path::new(&args().next_back().unwrap())).unwrap();

    day5(&mut source);

    //day2(&mut source);

    //let mut memory = Memory {
    //    bucket: RefCell::new(buf),
    //};

    //let mut intcode = IntCode {
    //    mem: &mut memory,
    //    ic: 0,
    //};

    //intcode.run_program();

    //println!("{}", intcode.mem.read(0));
}

fn day5(source: &mut File) {
    let mut buf = Vec::<i32>::new();
    load_program(&mut buf, source);

    let mut memory = Memory {
        bucket: RefCell::new(buf),
    };

    let mut intcode = IntCode {
        mem: &mut memory,
        ic: 0
    };

    intcode.run_program(Some(1));
}

fn day2(source: &mut File) {
    for x in 0..100 {
        for y in 0..100 {
            let mut buf = Vec::<i32>::new();
            load_program(&mut buf, source);

            let mut memory = Memory {
                bucket: RefCell::new(buf),
            };

            let mut intcode = IntCode {
                mem: &mut memory,
                ic: 0,
            };

            intcode.mem.write(1, x);
            intcode.mem.write(2, y);

            intcode.run_program(None);

            if intcode.mem.read(0) == 19690720 {
                println!("x: {}, y: {}", x, y);
                exit(0);
            }

            source.seek(SeekFrom::Start(0)).unwrap();
        }
    }
}

fn load_program(mem: &mut Vec<i32>, file: &mut File) {
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    let _ = content
        .as_mut_str()
        .trim_end()
        .split(',')
        .map(|x| {
            match x.parse::<i32>() {
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
        };

        assert_eq!(intcode.next(), Some(Instruction{
            op: OpCode::Add,
            args: [Some(10), Some(11), Some(12)],
            modes: [ParameterMode::Position, ParameterMode::Position, ParameterMode::Position],
            len: 4
        }));
    }

    #[test]
    fn get_multiple_instructions() {
        let mut memory = memory();
        let mut intcode = IntCode {
            mem: &mut memory,
            ic: 0,
        };

        intcode.next();

        assert_eq!(intcode.next(), Some(Instruction{
            op: OpCode::Mul,
            args: [Some(12), Some(10), Some(12)],
            modes: [ParameterMode::Position, ParameterMode::Position, ParameterMode::Position],
            len: 4
        }));
    }

    #[test]
    fn compound_opcode() {
        let mut memory = memory();
        let mut intcode = IntCode {
            mem: &mut memory,
            ic: 0
        };

        intcode.next();
        intcode.next();

        assert_eq!(intcode.next(), Some(Instruction{
            op: OpCode::Add,
            args: [Some(0), Some(1), Some(8)],
            modes: [ParameterMode::Position, ParameterMode::Position, ParameterMode::Immediate],
            len: 4
        }));
    }

    #[test]
    fn execution() {
        let mut memory = memory();
        let mut intcode = IntCode {
            mem: &mut memory,
            ic: 0,
        };

        let i = intcode.next().unwrap();
        intcode.execute(i, None);

        assert_eq!(intcode.mem.read(12), 3);
    }

    fn memory() -> Memory {
        Memory {
            bucket: RefCell::new(vec![1, 10, 11, 12, 2, 12, 10, 12, 101, 0, 1, 8, 99, 10, 3, 0, 0]),
        }
    }
}
