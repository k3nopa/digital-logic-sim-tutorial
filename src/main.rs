fn main() {
    println!("Hello, world!");
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
/// Copy is cheap because `Bit` Enum only repr as u8
enum Bit {
    Low,  // Low, 0
    High, // High, 1
}

trait Component {
    /// Update : means to update the output of the logic gate implemented
    fn update(&self, input: &[Bit]) -> Vec<Bit>;
}

struct Nand {
    num_inputs: usize,
}

impl Nand {
    fn new(num_inputs: usize) -> Self {
        Self { num_inputs }
    }
}

impl Component for Nand {
    fn update(&self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(self.num_inputs, input.len());
        /// If input is Low return High and vice versa
        if input.iter().any(|&a| a == Bit::Low) {
            vec![Bit::High]
        } else {
            vec![Bit::Low]
        }
    }
}

/// A or B == NOT(A) NAND NOT(B)
struct Or2 {
    nand_a: Nand,
    nand_b: Nand,
    nand_c: Nand,
}

impl Or2 {
    fn new() -> Self {
        Self {
            nand_a: Nand::new(1),
            nand_b: Nand::new(1),
            nand_c: Nand::new(2),
        }
    }
}

impl Component for Or2 {
    fn update(&self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(input.len(), 2);
        let a = input[0];
        let not_a = self.nand_a.update(&[a])[0];
        let b = input[1];
        let not_b = self.nand_a.update(&[b])[0];

        self.nand_c.update(&[not_a, not_b])
    }
}
