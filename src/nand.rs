use crate::{Bit, Component};

pub struct Nand {
    num_inputs: usize,
}

// Multiple inputs with 1 output
impl Nand {
    pub fn new(num_inputs: usize) -> Self {
        Self { num_inputs }
    }
}

impl Component for Nand {
    fn update(&mut self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(self.num_inputs, input.len());
        let mut x = Bit::L;
        // If input is L return H and vice versa
        for a in input {
            match *a {
                // If any input is 0. output is 1
                Bit::L => return vec![Bit::H],
                // X NAND L = H | X NAND H = X
                Bit::X => x = Bit::X,
                Bit::H => {}
            }
        }
        vec![x]
    }
    fn num_inputs(&self) -> usize {
        self.num_inputs
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn name(&self) -> &str {
        "NAND"
    }
}
