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
}

pub struct Point {
    pub x: f64,
    pub y: f64,
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
