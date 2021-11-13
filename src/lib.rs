pub mod nand;
pub mod or2;

pub use nand::Nand;
pub use or2::Or2;

use std::collections::HashMap;

static VCD_SHOW_NAND: bool = true;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
// Copy is cheap because `Bit` Enum only repr as u8
pub enum Bit {
    L, // Low, 0
    H, // High, 1
    X, // Undefined, Floating pins
}

impl From<Bit> for vcd::Value {
    fn from(x: Bit) -> Self {
        match x {
            Bit::L => vcd::Value::V0,
            Bit::H => vcd::Value::V1,
            Bit::X => vcd::Value::X,
        }
    }
}

pub trait Component {
    // Update : means to update the output of the logic gate implemented
    fn update(&mut self, input: &[Bit]) -> Vec<Bit>;
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn name(&self) -> &str;
    fn write_internal_components(
        &self,
        _w: &mut vcd::Writer<&mut dyn std::io::Write>,
        _i: &mut u64,
    ) -> std::io::Result<VcdSignalHandle> {
        Ok(VcdSignalHandle { id: HashMap::new() })
    }
    fn write_internal_signals(
        &self,
        _w: &mut vcd::Writer<&mut dyn std::io::Write>,
        _i: &mut u64,
        _vh: &VcdSignalHandle,
    ) -> std::io::Result<()> {
        Ok(())
    }
}

// Made from a bunch of Components(logic gate)
pub struct Structural {
    components: Vec<CompIo>,
    num_inputs: usize,
    num_outputs: usize,
    name: String,
}

impl Structural {
    pub fn new(components: Vec<CompIo>, num_inputs: usize, num_outputs: usize, name: &str) -> Self {
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
        let connections = self.components[c_id].connections.clone();
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
    fn update(&mut self, input: &[Bit]) -> Vec<Bit> {
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
    fn write_internal_components(
        &self,
        writer: &mut vcd::Writer<&mut dyn std::io::Write>,
        j: &mut u64,
    ) -> std::io::Result<VcdSignalHandle> {
        let mut vh = VcdSignalHandle { id: HashMap::new() };
        let write_parent = *j == 0;
        if write_parent {
            let mut vi = InstanceIndex::new(*j as usize, 0);
            let instance_name = format!("{}-{}", self.name(), j);
            writer.add_module(&instance_name)?;
            for i in 0..self.num_inputs {
                vh.id.insert(
                    vi,
                    writer.add_wire(1, &format!("{}-i{}", instance_name, i))?,
                );
                vi.port_id += 1;
            }
            for i in 0..self.num_outputs {
                vh.id.insert(
                    vi,
                    writer.add_wire(1, &format!("{}-o{}", instance_name, i))?,
                );
                vi.port_id += 1;
            }

            *j += 1;
        }

        for c in self
            .components
            .iter()
            .skip(1)
            .filter(|&c| VCD_SHOW_NAND || (c.comp.name() != "NAND"))
        {
            let mut vi = InstanceIndex::new(*j as usize, 0);
            let instance_name = format!("{}-{}", c.comp.name(), j);
            writer.add_module(&instance_name)?;
            for i in 0..c.comp.num_inputs() {
                vh.id.insert(
                    vi,
                    writer.add_wire(1, &format!("{}-i{}", instance_name, i))?,
                );
                vi.port_id += 1;
            }
            for i in 0..c.comp.num_outputs() {
                vh.id.insert(
                    vi,
                    writer.add_wire(1, &format!("{}-o{}", instance_name, i))?,
                );
                vi.port_id += 1;
            }
            *j += 1;
            let ch = c.comp.write_internal_components(writer, j)?;
            vh.id.extend(ch.id);
            writer.upscope()?;
        }

        if write_parent {
            writer.upscope()?;
        }
        Ok(vh)
    }

    fn write_internal_signals(
        &self,
        writer: &mut vcd::Writer<&mut dyn std::io::Write>,
        j: &mut u64,
        vh: &VcdSignalHandle,
    ) -> std::io::Result<()> {
        let write_parent = *j == 0;

        if write_parent {
            // TODO: create a less error prone helper method
            let ref inputs = self.components[0].output;
            let ref outputs = self.components[0].input;
            let vi = InstanceIndex::new(*j as usize, 0);
            write_vcd_signals(writer, vi, vh, inputs, outputs)?;
            *j += 1;
        }

        for c in self
            .components
            .iter()
            .skip(1)
            .filter(|&c| VCD_SHOW_NAND || (c.comp.name() != "NAND"))
        {
            let ref inputs = c.input;
            let ref outputs = c.output;
            let vi = InstanceIndex::new(*j as usize, 0);
            write_vcd_signals(writer, vi, vh, inputs, outputs)?;
            *j += 1;

            c.comp.write_internal_signals(writer, j, vh)?;
        }

        Ok(())
    }
}

// Component Input & Output info
pub struct CompIo {
    comp: Box<dyn Component>,
    input: Vec<Bit>,
    output: Vec<Bit>,
    // [output_id, [component_id, input_id]]
    connections: Vec<Vec<Index>>,
}

impl CompIo {
    pub fn new(comp: Box<dyn Component>) -> Self {
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
    // Logic gate that will be use for outputing signal for whole Structural Component
    pub fn c_zero(num_inputs: usize, num_outputs: usize) -> Self {
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
    pub fn add_connection(&mut self, output_id: usize, to: Index) {
        self.connections[output_id].push(to);
    }
}

#[derive(Clone)]
pub struct Index {
    comp_id: usize,
    input_id: usize,
}

impl Index {
    pub fn new(comp_id: usize, input_id: usize) -> Self {
        Self { comp_id, input_id }
    }
}

pub struct VcdSignalHandle {
    id: HashMap<InstanceIndex, vcd::IdCode>,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct InstanceIndex {
    instance_id: usize,
    port_id: usize,
}

impl InstanceIndex {
    fn new(instance_id: usize, port_id: usize) -> Self {
        Self { instance_id, port_id }
    }
}

fn write_vcd_signals(
    writer: &mut vcd::Writer<&mut dyn std::io::Write>,
    vi: InstanceIndex,
    vh: &VcdSignalHandle,
    signals1: &[Bit],
    signals2: &[Bit],
) -> std::io::Result<InstanceIndex> {
    let mut vi = vi.clone();

    for s in signals1 {
        let h = vh.id[&vi];
        writer.change_scalar(h, *s)?;
        vi.port_id += 1;
    }

    for s in signals2 {
        let h = vh.id[&vi];
        writer.change_scalar(h, *s)?;
        vi.port_id += 1;
    }

    Ok(vi)
}

pub fn run_simulation(
    w: &mut dyn std::io::Write,
    c: &mut dyn Component,
    inputs: &[Vec<Bit>],
    ticks: usize,
) -> std::io::Result<()> {
    let mut writer = vcd::Writer::new(w);
    // Header: 1 tick = 1ns
    writer.timescale(1, vcd::TimescaleUnit::NS)?;
    let vh = c.write_internal_components(&mut writer, &mut 0)?;
    writer.add_module(&format!("clk"))?;
    let clk = writer.add_wire(1, "clk")?;
    writer.upscope()?;

    writer.enddefinitions()?;

    // Write initial values
    writer.begin(vcd::SimulationCommand::Dumpvars)?;
    writer.change_scalar(clk, Bit::L)?;

    // Initialize everything to X
    for h in vh.id.values() {
        writer.change_scalar(*h, Bit::X)?;
    }
    writer.end()?;

    // Update the components and signals in a loop
    let mut clk_on = true;
    for t in 0..ticks {
        writer.timestamp(t as u64)?;
        let _ = c.update(&inputs[t]);
        c.write_internal_signals(&mut writer, &mut 0, &vh)?;
        writer.change_scalar(
            clk,
            if clk_on {
                vcd::Value::V1
            } else {
                vcd::Value::V0
            },
        )?;
        clk_on = !clk_on;
    }
    writer.timestamp(ticks as u64)?;
    Ok(())
}
