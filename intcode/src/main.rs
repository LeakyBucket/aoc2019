extern crate itertools;

use itertools::Itertools;

use std::cell::RefCell;
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::process::exit;

#[derive(Debug)]
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
        let mut parts: [i32;3] = [0;3];

        for pos in 2..5 {
            let div = 10_i32.pow(6 - pos);
            let mode = label/div;

            parts[(pos - 2) as usize] = mode;

            label = label - (mode * div);
        }

        (label, parts[2], parts[1], parts[0])
    }

    fn len(op: &OpCode) -> usize {
        match op {
            OpCode::Add => 4,
            OpCode::Mul => 4,
            OpCode::LessThan => 4,
            OpCode::Equals => 4,
            OpCode::Halt => 1,
            OpCode::Input => 2,
            OpCode::Output => 2,
            OpCode::JumpIfTrue => 3,
            OpCode::JumpIfFalse => 3,
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
                // Incrementing the program counter here is fine because the instruction
                // is ecexuted afterwords, this means we don't mess with our jump addresses.
                self.ic = self.ic + instruction.len;
                Some(instruction)
            },
        }
    }
}

impl IntCode<'_> {
    fn run_program(&mut self, input: &mut Vec<i32>) -> Vec<i32> {
        let mut output = Vec::<i32>::new();

        while let Some(i) = self.next() {
            if let Some(result) = self.execute(i, input) {
                output.push(result);
            };
        }

        output
    }

    fn execute(&mut self, i: Instruction, input: &mut Vec<i32>) -> Option<i32> {
        let mut output = None;

        match i.op {
            OpCode::Add => self.add(i),
            OpCode::Mul => self.mul(i),
            OpCode::Input => self.mem.write(i.args[0].unwrap() as usize, input.remove(0)),
            OpCode::Output => output = Some(self.mem.read(i.args[0].unwrap() as usize)),
            OpCode::JumpIfFalse => self.jump_if_false(i),
            OpCode::JumpIfTrue => self.jump_if_true(i),
            OpCode::Equals => self.equal(i),
            OpCode::LessThan => self.less_than(i),
            _ => (),
        }

        output
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

    fn jump_if_true(&mut self, i: Instruction) {
        let op1 = match i.modes[0] {
            ParameterMode::Immediate => i.args[0].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[0].unwrap() as usize),
        };
        let op2 = match i.modes[1] {
            ParameterMode::Immediate => i.args[1].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[1].unwrap() as usize),
        };

        if op1 != 0 { self.ic = op2 as usize; }
    }

    fn jump_if_false(&mut self, i: Instruction) {
        let op1 = match i.modes[0] {
            ParameterMode::Immediate => i.args[0].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[0].unwrap() as usize),
        };
        let op2 = match i.modes[1] {
            ParameterMode::Immediate => i.args[1].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[1].unwrap() as usize),
        };

        if op1 == 0 { self.ic = op2 as usize;}
    }

    fn less_than(&self, i: Instruction) {
        let op1 = match i.modes[0] {
            ParameterMode::Immediate => i.args[0].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[0].unwrap() as usize),
        };
        let op2 = match i.modes[1] {
            ParameterMode::Immediate => i.args[1].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[1].unwrap() as usize),
        };
        let op3 = i.args[2].unwrap();

        if op1 < op2 {
            self.mem.write(op3 as usize, 1);
        } else {
            self.mem.write(op3 as usize, 0);
        }
    }

    fn equal(&self, i: Instruction) {
        let op1 = match i.modes[0] {
            ParameterMode::Immediate => i.args[0].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[0].unwrap() as usize),
        };
        let op2 = match i.modes[1] {
            ParameterMode::Immediate => i.args[1].unwrap(),
            ParameterMode::Position => self.mem.read(i.args[1].unwrap() as usize),
        };
        let op3 = i.args[2].unwrap();

        if op1 == op2 {
            self.mem.write(op3 as usize, 1);
        } else {
            self.mem.write(op3 as usize, 0);
        }
    }
}

fn main() {
    let mut source = File::open(Path::new(&args().next_back().unwrap())).unwrap();

    day7(&mut source);

    //day5(&mut source, 1);

    //day5(&mut source, 5);

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

fn day7(source: &mut File) {
    let mut buf = Vec::<i32>::new();
    load_program(&mut buf, source);

    let inputs: Vec<Vec<i32>> = (0..5).permutations(5).collect();

    let max = inputs.iter().map(|i| {
        amplifier_sequence(&i, &buf)
    }).max();

    dbg!(max);

    //for i in 0..5 {
    //    (i + 5) % 5
    //}
}

fn amplifier_sequence(seq: &Vec<i32>, mem: &Vec<i32>) -> i32 {
    let mut output = 0;

    for x in 0..5 {
        let mut memory = Memory {
            bucket: RefCell::new(mem.clone()),
        };

        let mut intcode = IntCode {
            mem: &mut memory,
            ic: 0
        };

        let mut args: Vec<i32> = Vec::new();

        args.push(seq[x]);
        args.push(output);
        output = intcode.run_program(&mut args)[0];
        //if let o = output {
        //    args.push(seq[x]);
        //    args.push(o);
        //    dbg!(&args);
        //    output = intcode.run_program(&mut args)[0];
        //} else {
        //    args.push(seq[x]);
        //    args.push(0);
        //    dbg!(&args);
        //    output = Some(intcode.run_program(&mut args)[0]);
        //}
    }

    output
}

fn day5(source: &mut File, mod_id: i32) {
    let mut buf = Vec::<i32>::new();
    load_program(&mut buf, source);

    let mut memory = Memory {
        bucket: RefCell::new(buf),
    };

    let mut intcode = IntCode {
        mem: &mut memory,
        ic: 0
    };

    let mut inputs = vec![mod_id];

    dbg!(intcode.run_program(&mut inputs));
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

            let mut inputs = vec![];

            intcode.mem.write(1, x);
            intcode.mem.write(2, y);

            intcode.run_program(&mut inputs);

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
            modes: [ParameterMode::Immediate, ParameterMode::Position, ParameterMode::Position],
            len: 4
        }));
    }

    #[test]
    fn execution() {
        let mut memory = Memory {
            bucket: RefCell::new(vec![3,9,8,9,10,9,4,9,99,-1,8]),
        };
        let mut intcode = IntCode {
            mem: &mut memory,
            ic: 0,
        };

        let mut args = vec![7];

        assert_eq!(intcode.run_program(&mut args), vec![0]);
    }

    #[test]
    fn amplifier_long() {
        let buf = vec![3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0];
        let seq = vec![1,0,4,3,2];
        let result = amplifier_sequence(&seq, &buf);

        assert_eq!(result, 65210);
    }

    #[test]
    fn amplifier_short() {
        // 17 elements, last position 16
        let buf = vec![3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0];
        let seq = vec![4,3,2,1,0];
        let result = amplifier_sequence(&seq, &buf);

        assert_eq!(result, 43210);
    }

    #[test]
    fn amplifier_medium() {
        let buf = vec![3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0];
        let seq = vec![0,1,2,3,4];
        let result = amplifier_sequence(&seq, &buf);

        assert_eq!(result, 54321);
    }

    #[test]
    fn feedback_small() {
        let buf = vec![3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5];
        let seq = vec![9,8,7,6,5];
        let result = amplifier_sequence(&seq, &buf);

        assert_eq!(result, 139629729);
    }

    fn memory() -> Memory {
        Memory {
            bucket: RefCell::new(vec![1, 10, 11, 12, 2, 12, 10, 12, 101, 0, 1, 8, 99, 10, 3, 0, 0]),
        }
    }
}
