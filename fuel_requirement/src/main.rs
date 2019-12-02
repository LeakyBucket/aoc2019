use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

fn main() {
    let mut fuel: u64 = 0;
    let sizes = File::open(Path::new(&args().next_back().unwrap())).unwrap();

    for line in BufReader::new(sizes).lines() {
        if let Ok(s) = line {
            fuel = fuel + fuel_requirement(s.parse::<u64>().unwrap())
        }
    }

    println!("{}", fuel)
}

fn fuel_requirement(mod_size: u64) -> u64 {
    let base = (mod_size / 3) - 2;

    meta_fuel(base, base)
}

fn meta_fuel(mass: u64, total: u64) -> u64 {
    let fraction = mass / 3;

    if fraction < 2 {
        total
    } else {
        meta_fuel(fraction - 2, total + fraction - 2)
    }
}