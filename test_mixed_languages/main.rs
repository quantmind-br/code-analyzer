// Rust file
fn main() {
    println!("Hello from Rust!");
}

struct Calculator {
    value: i32,
}

impl Calculator {
    fn new(value: i32) -> Self {
        Self { value }
    }
    
    fn add(&mut self, n: i32) -> &mut Self {
        self.value += n;
        self
    }
}