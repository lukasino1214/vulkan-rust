pub struct Model {
    bruh: i32
}

impl Model {
    pub fn new() -> Self{
        Self {
            bruh: 1
        }
    }

    pub fn render(&self) {
        println!("test");
    }
}