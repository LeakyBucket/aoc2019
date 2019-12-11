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
    bucket: RefCell<Vec<i64>>,
}

impl Memory {
    fn read(&self, index: usize) -> i64 {
        if self.bucket.borrow().len() <= index {
            0
        } else {
            self.bucket.borrow()[index]
        }
    }

    fn write(&self, index: usize, value: i64) {
        if self.bucket.borrow().len() <= index {
            self.bucket.borrow_mut().resize_with(index + 1, {|| 0 as i64});
            self.bucket.borrow_mut()[index] = value;
        } else {
            self.bucket.borrow_mut()[index] = value;
        }
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
    RelativeBase,
    Unknown,
}

impl From<i64> for OpCode {
    fn from(i: i64) -> OpCode {
        match i {
            1 => OpCode::Add,
            2 => OpCode::Mul,
            3 => OpCode::Input,
            4 => OpCode::Output,
            5 => OpCode::JumpIfTrue,
            6 => OpCode::JumpIfFalse,
            7 => OpCode::LessThan,
            8 => OpCode::Equals,
            9 => OpCode::RelativeBase,
            99 => OpCode::Halt,
            _ => OpCode::Unknown,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ParameterMode {
    Position,
    Immediate,
    Relative,
}

impl From<i64> for ParameterMode {
    fn from(f: i64) -> ParameterMode {
        match f {
            2 => ParameterMode::Relative,
            1 => ParameterMode::Immediate,
            _ => ParameterMode::Position,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Instruction {
    op: OpCode,
    pub args: [Option<i64>;3],
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
    fn new(label: i64) -> Self {
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

    fn process_label(label: i64) -> (i64, i64, i64, i64) {
        let mut label = label;
        let mut parts: [i64;3] = [0;3];

        for pos in 2..5 {
            let div = 10_i64.pow(6 - pos);
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
            OpCode::RelativeBase => 2,
            _ => 0
        }
    }
}

#[derive(Debug)]
struct IntCode {
    mem: Memory,
    ic: usize,
    relative_base: usize,
}

impl Iterator for IntCode {
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

impl IntCode {
    fn new(mem: Memory) -> Self {
        IntCode {
            mem,
            ic: 0,
            relative_base: 0,
        }
    }

    fn run_program(&mut self, input: &mut Vec<i64>) -> Vec<i64> {
        let mut output = Vec::<i64>::new();

        while let Some(i) = self.next() {
            if let Some(result) = self.execute(i, input) {
                output.push(result);
            };
        }

        output
    }

    fn execute(&mut self, i: Instruction, input: &mut Vec<i64>) -> Option<i64> {
        let mut output = None;

        match i.op {
            OpCode::Add => self.add(i),
            OpCode::Mul => self.mul(i),
            OpCode::Input => self.input(i, input),
            OpCode::Output => output = self.output(i),
            OpCode::JumpIfFalse => self.jump_if_false(i),
            OpCode::JumpIfTrue => self.jump_if_true(i),
            OpCode::Equals => self.equal(i),
            OpCode::LessThan => self.less_than(i),
            OpCode::RelativeBase => self.relative_inc(i),
            _ => (),
        }

        output
    }

    fn input(&self, i: Instruction, inputs: &mut Vec<i64>) {
        let op1 = match &i.modes[0] {
            ParameterMode::Relative => {
                let pos = self.relative_base as i64 + i.args[0].unwrap();
                pos as usize
            },
            _ => i.args[0].unwrap() as usize,
        };

        self.mem.write(op1, inputs.remove(0));
    }

    fn output(&self, i: Instruction) -> Option<i64> {
        let op1 = self.value(i.args[0].unwrap(), &i.modes[0]);

        Some(op1)
    }

    fn add(&self, i: Instruction) {
        let op1 = self.value(i.args[0].unwrap(), &i.modes[0]);
        let op2 = self.value(i.args[1].unwrap(), &i.modes[1]);
        let op3 = match i.modes[2] {
            ParameterMode::Relative => i.args[2].unwrap() + self.relative_base as i64,
            _ => i.args[2].unwrap(),
        };

        self.mem.write(op3 as usize, op1 + op2);
    }

    fn mul(&self, i: Instruction) {
        let op1 = self.value(i.args[0].unwrap(), &i.modes[0]);
        let op2 = self.value(i.args[1].unwrap(), &i.modes[1]);
        let op3 = match i.modes[2] {
            ParameterMode::Relative => i.args[2].unwrap() + self.relative_base as i64,
            _ => i.args[2].unwrap(),
        };

        self.mem.write(op3 as usize, op1 * op2);
    }

    fn jump_if_true(&mut self, i: Instruction) {
        let op1 = self.value(i.args[0].unwrap(), &i.modes[0]);
        let op2 = self.value(i.args[1].unwrap(), &i.modes[1]);

        if op1 != 0 { self.ic = op2 as usize; }
    }

    fn jump_if_false(&mut self, i: Instruction) {
        let op1 = self.value(i.args[0].unwrap(), &i.modes[0]);
        let op2 = self.value(i.args[1].unwrap(), &i.modes[1]);

        if op1 == 0 { self.ic = op2 as usize; }
    }

    fn less_than(&self, i: Instruction) {
        let op1 = self.value(i.args[0].unwrap(), &i.modes[0]);
        let op2 = self.value(i.args[1].unwrap(), &i.modes[1]);
        let op3 = match i.modes[2] {
            ParameterMode::Relative => i.args[2].unwrap() + self.relative_base as i64,
            _ => i.args[2].unwrap(),
        };

        if op1 < op2 {
            self.mem.write(op3 as usize, 1);
        } else {
            self.mem.write(op3 as usize, 0);
        }
    }

    fn equal(&self, i: Instruction) {
        let op1 = self.value(i.args[0].unwrap(), &i.modes[0]);
        let op2 = self.value(i.args[1].unwrap(), &i.modes[1]);
        let op3 = match i.modes[2] {
            ParameterMode::Relative => i.args[2].unwrap() + self.relative_base as i64,
            _ => i.args[2].unwrap(),
        };

        if op1 == op2 {
            self.mem.write(op3 as usize, 1);
        } else {
            self.mem.write(op3 as usize, 0);
        }
    }

    fn relative_inc(&mut self, i: Instruction) {
        let op1 = self.value(i.args[0].unwrap(), &i.modes[0]);
        let new_base = self.relative_base as i64 + op1;

        self.relative_base = new_base as usize;
    }

    fn value(&self, op: i64, pm: &ParameterMode) -> i64 {
        match pm {
            ParameterMode::Immediate => op,
            ParameterMode::Position => self.mem.read(op as usize),
            ParameterMode::Relative => {
                let position = op + self.relative_base as i64;
                self.mem.read(position as usize)
            }
        }
    }
}

struct Amplifier {
    phase: i64,
    cpu: IntCode,
}

impl Amplifier {
    fn new(phase: i64, cpu: IntCode) -> Self {
        Amplifier {
            phase,
            cpu,
        }
    }

    fn run(&mut self, input: i64) -> Option<i64> {
        let mut result = None;
        let mut input = if self.cpu.ic == 0 {
            vec![self.phase, input]
        } else {
            vec![input]
        };

        while let Some(i) = self.cpu.next() {
            match i.op {
                OpCode::Output => {
                    result = self.cpu.execute(i, &mut input);
                    break;
                },
                _ => {
                    self.cpu.execute(i, &mut input);
                }
            }
        }

        result
    }
}

fn main() {
    let mut source = File::open(Path::new(&args().next_back().unwrap())).unwrap();

    day9(&mut source);

    //day7(&mut source);

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

fn day9(source: &mut File) {
    let mut buf = Vec::<i64>::new();
    load_program(&mut buf, source);

    let mem = Memory {
        bucket: RefCell::new(buf),
    };

    let mut intcode = IntCode::new(mem);
    let mut input: Vec<i64> = vec![2];

    dbg!(intcode.run_program(&mut input));
}

fn day7(source: &mut File) {
    let mut buf = Vec::<i64>::new();
    load_program(&mut buf, source);

    let inputs: Vec<Vec<i64>> = (0..5).permutations(5).collect();

    let max = inputs.iter().map(|i| {
        amplifier_sequence(&i, &buf)
    }).max();

    dbg!(max);

    let inputs: Vec<Vec<i64>> = (5..10).permutations(5).collect();

    let max = inputs.iter().map(|i| {
        feedback(&i, &buf)
    }).max();

    println!("Feedback: {}", max.unwrap());

    //for i in 0..5 {
    //    (i + 5) % 5
    //}
}

fn feedback(seq: &Vec<i64>, mem: &Vec<i64>) -> i64 {
    let mut value = 0;
    let mut amps = Vec::<Amplifier>::new();

    for i in seq.iter() {
        let memory = Memory {
            bucket: RefCell::new(mem.clone()),
        };
        let intcode = IntCode::new(memory);
        let amp = Amplifier::new(*i, intcode);

        amps.push(amp);
    }

    while amps.len() > 0 {
        for x in 0..5 {
            if amps.len() > x {
                match amps[x].run(value) {
                    Some(v) => value = v,
                    None => {
                        amps.remove(x);
                    }
                }
            }
        }
    }

    value
}

fn amplifier_sequence(seq: &Vec<i64>, mem: &Vec<i64>) -> i64 {
    let mut output = 0;

    for x in 0..5 {
        let memory = Memory {
            bucket: RefCell::new(mem.clone()),
        };
        let mut intcode = IntCode::new(memory);
        let mut args: Vec<i64> = Vec::new();

        dbg!(&intcode.mem);

        args.push(seq[x]);
        args.push(output);
        output = intcode.run_program(&mut args)[0];
    }

    output
}

fn day5(source: &mut File, mod_id: i64) {
    let mut buf = Vec::<i64>::new();
    load_program(&mut buf, source);

    let memory = Memory {
        bucket: RefCell::new(buf),
    };
    let mut intcode = IntCode::new(memory);
    let mut inputs = vec![mod_id];

    dbg!(intcode.run_program(&mut inputs));
}

fn day2(source: &mut File) {
    for x in 0..100 {
        for y in 0..100 {
            let mut buf = Vec::<i64>::new();
            load_program(&mut buf, source);

            let memory = Memory {
                bucket: RefCell::new(buf),
            };
            let mut intcode = IntCode::new(memory);
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

fn load_program(mem: &mut Vec<i64>, file: &mut File) {
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    let _ = content
        .as_mut_str()
        .trim_end()
        .split(',')
        .map(|x| {
            match x.parse::<i64>() {
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
        let memory = memory();
        let mut intcode = IntCode::new(memory);

        assert_eq!(intcode.next(), Some(Instruction{
            op: OpCode::Add,
            args: [Some(10), Some(11), Some(12)],
            modes: [ParameterMode::Position, ParameterMode::Position, ParameterMode::Position],
            len: 4
        }));
    }

    #[test]
    fn get_multiple_instructions() {
        let memory = memory();
        let mut intcode = IntCode::new(memory);

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
        let memory = memory();
        let mut intcode = IntCode::new(memory);

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
        let memory = Memory {
            bucket: RefCell::new(vec![3,9,8,9,10,9,4,9,99,-1,8]),
        };
        let mut intcode = IntCode::new(memory);
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
        let result = feedback(&seq, &buf);

        assert_eq!(result, 139629729);
    }

    #[test]
    fn boost_all() {
        let buf = vec![109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99];
        let memory = Memory {
            bucket: RefCell::new(buf.clone()),
        };
        let mut intcode = IntCode::new(memory);
        let mut input = vec![1];
        let output = intcode.run_program(&mut input);

        assert_eq!(output, buf);
    }

    #[test]
    fn boost_long_number() {
        let buf = vec![1102,34915192,34915192,7,4,7,99,0];
        let memory = Memory {
            bucket: RefCell::new(buf.clone()),
        };
        let mut intcode = IntCode::new(memory);
        let mut input = vec![1];
        let output = intcode.run_program(&mut input);

        assert_eq!(output[0], 1219070632396864);
    }

    #[test]
    fn boost_middle_number() {
        let buf = vec![104,1125899906842624,99];
        let memory = Memory {
            bucket: RefCell::new(buf.clone()),
        };
        let mut intcode = IntCode::new(memory);
        let mut input = vec![1];
        let output = intcode.run_program(&mut input);

        assert_eq!(output[0], 1125899906842624);
    }


    fn memory() -> Memory {
        Memory {
            bucket: RefCell::new(vec![1, 10, 11, 12, 2, 12, 10, 12, 101, 0, 1, 8, 99, 10, 3, 0, 0]),
        }
    }
}
