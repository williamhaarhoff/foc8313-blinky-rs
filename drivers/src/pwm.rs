pub struct Pwm {
    pub id: u8,
}

impl Pwm {
    pub fn new(id: u8) -> Self {
        Self { id }
    }

    pub fn read(&self) -> f32 {
        // Dummy implementation
        42.0
    }
}
