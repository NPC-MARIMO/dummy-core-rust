#[derive(Debug)]
pub struct ExpSmoother {
    alpha: f32,
    value: Option<f32>,
}

impl ExpSmoother {
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha,
            value: None,
        }
    }

    pub fn update(&mut self, input: f32) -> f32 {
        let output = match self.value {
            Some(prev) => self.alpha * input + (1.0 - self.alpha) * prev,
            None => input,
        };

        self.value = Some(output);
        output
    }
}