use std::collections::{HashMap, HashSet};
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, PartialEq)]
struct Point(i16, i16);

impl Point {
    fn up(&mut self) {
        self.1 = self.1 + 1;
    }

    fn down(&mut self) {
        self.1 = self.1 - 1;
    }

    fn left(&mut self) {
        self.0 = self.0 - 1;
    }

    fn right(&mut self) {
        self.0 = self.0 + 1;
    }
}

#[derive(Debug)]
struct WirePath {
    vertices: HashMap<i16, HashSet<i16>>,
    steps: Vec<Point>,
    pos: Point,
}

impl WirePath {
    fn new() -> Self {
        WirePath {
            vertices: HashMap::new(),
            steps: Vec::<Point>::new(),
            pos: Point(0, 0),
        }
    }

    fn record(&mut self, x: i16, y: i16) {
        match self.vertices.get_mut(&x) {
            Some(set) => {
                set.insert(y);
                ()
            }
            None => {
                let mut set = HashSet::new();
                set.insert(y);
                self.vertices.insert(x, set);
                ()
            }
        }
    }

    fn up(&mut self) {
        self.pos.up();
        self.steps.push(Point(self.pos.0, self.pos.1));
        self.record(self.pos.0, self.pos.1);
    }

    fn down(&mut self) {
        self.pos.down();
        self.steps.push(Point(self.pos.0, self.pos.1));
        self.record(self.pos.0, self.pos.1);
    }

    fn right(&mut self) {
        self.pos.right();
        self.steps.push(Point(self.pos.0, self.pos.1));
        self.record(self.pos.0, self.pos.1);
    }

    fn left(&mut self) {
        self.pos.left();
        self.steps.push(Point(self.pos.0, self.pos.1));
        self.record(self.pos.0, self.pos.1);
    }

    fn intersections(&self, other: &WirePath) -> Option<Vec<Point>> {
        let mut intersections = Vec::<Point>::new();

        for x in self.vertices.keys() {
            let cols = self.vertices.get(x).unwrap();

            match other.vertices.get(x) {
                Some(other_cols) => {
                    let _ = cols
                        .intersection(other_cols)
                        .map(|y| {
                            intersections.push(Point(*x, *y));
                            *y
                        })
                        .collect::<Vec<i16>>();
                }
                None => (),
            }
        }

        match intersections.len() {
            0 => None,
            _ => Some(intersections),
        }
    }
}

// rows for keys and sets of columns, one for each wire, then check shared rows for column intersections

fn main() {
    let mut wires = Vec::<WirePath>::new();
    let diagram = File::open(Path::new(&args().next_back().unwrap())).unwrap();

    for line in BufReader::new(diagram).lines() {
        let mut wire = WirePath::new();

        let _ = line
            .unwrap()
            .as_mut_str()
            .trim_end()
            .split(',')
            .map(|i| {
                let (dir, dist) = i.split_at(1);
                let dist = dist.parse::<i16>().unwrap();
                match dir {
                    "R" => {
                        for _ in 0..dist {
                            wire.right();
                        }
                    }
                    "U" => {
                        for _ in 0..dist {
                            wire.up();
                        }
                    }
                    "L" => {
                        for _ in 0..dist {
                            wire.left();
                        }
                    }
                    "D" => {
                        for _ in 0..dist {
                            wire.down()
                        }
                    }
                    _ => (),
                }
                dir
            })
            .collect::<Vec<&str>>();

        wires.push(wire);
    }

    let distances = wires[0]
        .intersections(&wires[1])
        .unwrap()
        .iter()
        .map(|point| point.0.abs() + point.1.abs())
        .min();

    println!("Closest: {}", distances.unwrap());
    println!("Shortest Path: {}", closest_intersection(wires));
}

fn closest_intersection(paths: Vec<WirePath>) -> usize {
    let intersections = paths[0].intersections(&paths[1]).unwrap();

    intersections
        .iter()
        .map(|point| {
            paths[0].steps.iter().position(|p| p == point).unwrap()
                + paths[1].steps.iter().position(|p| p == point).unwrap()
                + 2
        })
        .min()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIRING: &'static str = "R3,U2,L6\nU4,R6,U9\n";

    #[test]
    fn intersections() {}
}
