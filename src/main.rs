fn main() {
    // test NAND Component
    let nand = Nand::new(2);
    let output = nand.update(&[Bit::X, Bit::H]);
    println!("Nand Output: {:?}", output);

    // test OR Component
    let or = Or2::new();
    let output = or.update(&[Bit::X, Bit::H]);
    println!("OR Output: {:?}", output);
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
// Copy is cheap because `Bit` Enum only repr as u8
enum Bit {
    L, // Low, 0
    H, // High, 1
    X, // Undefined, Floating pins
}

trait Component {
    // Update : means to update the output of the logic gate implemented
    fn update(&self, input: &[Bit]) -> Vec<Bit>;
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn name(&self) -> &str;
}

struct Nand {
    num_inputs: usize,
}

// Multiple inputs with 1 output
impl Nand {
    fn new(num_inputs: usize) -> Self {
        Self { num_inputs }
    }
}

impl Component for Nand {
    fn update(&self, input: &[Bit]) -> Vec<Bit> {
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

struct Or2 {
    in_a: Nand,
    in_b: Nand,
    out: Nand,
}

// A or B == NOT(A) NAND NOT(B)
impl Or2 {
    fn new() -> Self {
        Self {
            in_a: Nand::new(1),
            in_b: Nand::new(1),
            out: Nand::new(2),
        }
    }
}

impl Component for Or2 {
    fn update(&self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(input.len(), 2);
        let a = input[0];
        let not_a = self.in_a.update(&[a])[0];
        let b = input[1];
        let not_b = self.in_b.update(&[b])[0];

        self.out.update(&[not_a, not_b])
    }
    fn num_inputs(&self) -> usize {
        todo!()
    }
    fn num_outputs(&self) -> usize {
        todo!()
    }
    fn name(&self) -> &str {
        todo!()
    }
}

// Made of a bunch of Components
struct Structural {
    components: Vec<CompIo>,
    num_inputs: usize,
    num_outputs: usize,
    name: String,
}

impl Structural {
    fn new(components: Vec<CompIo>, num_inputs: usize, num_outputs: usize, name: &str) -> Self {
        // Check that component_id 0 must be c_zero.
        // Because c_zero is the final output of Structural,
        // it's IO are supposed to be same as given parameter.
        assert_eq!(components[0].input.len(), num_outputs);
        assert_eq!(components[0].output.len(), num_inputs);
        assert_eq!(components[0].connections.len(), num_inputs);

        // TODO: check all the connections are valid
        let name = name.to_string();
        Self {
            components,
            num_inputs,
            num_outputs,
            name,
        }
    }

    fn propagate(&mut self, c_id: usize) {
        // TODO: avoid this clone
        let connections = self.components[c_id].connections;
        for (out_id, to) in connections.iter().enumerate() {
            for i in to {
                self.components[i.comp_id].input[i.input_id] = self.components[c_id].output[out_id];
            }
        }
    }

    fn propagate_input(&mut self, input: &[Bit]) {
        // The input is an output when seen from inside
        self.components[0].output = input.to_vec();
        self.propagate(0);
    }

    fn propagate_signals(&mut self) {
        // Propagate signals for every components
        for c in 1..self.components.len() {
            self.propagate(c);
        }
    }

    fn output(&self) -> Vec<Bit> {
        self.components[0].input.clone()
    }

    fn update_components(&mut self) {
        for c in 1..self.components.len() {
            let CompIo {
                ref mut comp,
                ref input,
                ref mut output,
                connections: _,
            } = self.components[c];
            *output = comp.update(input);
        }
    }
}

impl Component for Structural {
    // Steps below are important to make sure we got intended results.
    // In order to avoid updating a logic gate before input signal come in.
    // 1. Propagate input: so the components have the correct input when updated
    // 2. Update components: but don’t propagate the updates yet
    // 3. Propagate signals: after updating all the components, propagate the changes
    fn update(&self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(input.len(), self.num_inputs());
        // Propagate input
        self.propagate_input(input);
        // Update components
        self.update_components();
        // Propagate signal
        self.propagate_signals();
        // Return the component output
        self.output()
    }

    fn num_inputs(&self) -> usize {
        self.num_inputs
    }

    fn num_outputs(&self) -> usize {
        self.num_outputs
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// Component Input & Output info
struct CompIo {
    comp: Box<dyn Component>,
    input: Vec<Bit>,
    output: Vec<Bit>,
    // [output_id, [component_id, input_id]]
    connections: Vec<Vec<Index>>,
}

impl CompIo {
    fn new(comp: Box<dyn Component>) -> Self {
        let input = vec![Bit::X; comp.num_inputs()];
        let output = vec![Bit::X; comp.num_outputs()];
        let connections = vec![vec![]; comp.num_outputs()];
        Self {
            comp,
            input,
            output,
            connections,
        }
    }
    fn c_zero(num_inputs: usize, num_outputs: usize) -> Self {
        let comp = Box::new(Nand::new(0));
        let input = vec![Bit::X; num_outputs];
        let output = vec![Bit::X; num_inputs];
        let connections = vec![vec![]; num_inputs];
        Self {
            comp,
            input,
            output,
            connections,
        }
    }
    // Add the output from Self component to Index's component's input.
    fn add_connection(&mut self, output_id: usize, to: Index) {
        self.connections[output_id].push(to);
    }
}

#[derive(Clone)]
struct Index {
    comp_id: usize,
    input_id: usize,
}
