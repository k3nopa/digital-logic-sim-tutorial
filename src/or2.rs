use crate::nand::Nand;
use crate::{Bit, Component};

pub struct Or2 {
    in_a: Nand,
    in_b: Nand,
    out: Nand,
}

// A or B == NOT(A) NAND NOT(B)
impl Or2 {
    pub fn new() -> Self {
        Self {
            in_a: Nand::new(1),
            in_b: Nand::new(1),
            out: Nand::new(2),
        }
    }
}

impl Component for Or2 {
    fn update(&mut self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(input.len(), 2);
        let a = input[0];
        let not_a = self.in_a.update(&[a])[0];
        let b = input[1];
        let not_b = self.in_b.update(&[b])[0];

        self.out.update(&[not_a, not_b])
    }
    fn num_inputs(&self) -> usize {
        2
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn name(&self) -> &str {
        "OR2"
    }
}
