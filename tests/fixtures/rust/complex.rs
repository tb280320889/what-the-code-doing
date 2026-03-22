use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn add(&mut self, a: i32, b: i32) -> i32 {
        self.value = a + b;
        self.value
    }

    pub fn print(&self, message: &str) {
        println!("{}", message);
    }
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

pub trait Processor {
    fn process(&self, data: &str) -> Result<(), Error>;
}

pub enum Color {
    Red,
    Green,
    Blue,
}

pub type StringMap = HashMap<String, String>;

pub const MAX_SIZE: usize = 100;

static mut COUNTER: u32 = 0;

mod utils;

fn main() {
    let mut calc = Calculator::new();
    let result = calc.add(5, 10);
    println!("{}", result);
}
