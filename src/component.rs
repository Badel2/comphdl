use vcd;
use bit::Bit;
use parser::{CompInfo, CompDefinition};
use std;
use std::io;
use std::collections::HashMap;
use std::rc::Rc;

static VCD_SHOW_NAND: bool = true;

pub trait Component: std::fmt::Debug {
    fn update(&mut self, input: &[Bit]) -> Vec<Bit>;
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn name(&self) -> &str;
    fn write_internal_components(&self, _w: &mut vcd::Writer, _i: &mut u64) -> io::Result<VcdSignalHandle> {
        Ok(VcdSignalHandle { id: HashMap::new() })
    }
    fn write_internal_signals(&self, _w: &mut vcd::Writer, _i: &mut u64, _vh: &VcdSignalHandle) -> io::Result<()> {
        Ok(())
    }
    fn port_names(&self) -> PortNames {
        PortNames::default(self.num_inputs(), self.num_outputs())
    }
    fn clone_as_structural(&self) -> Option<Structural> {
        Some(Structural::new_wrap(self.box_clone()))
    }
    fn box_clone(&self) -> Box<Component>;
}

impl Clone for Box<Component> {
    fn clone(&self) -> Box<Component> {
        self.box_clone()
    }
}

#[derive(Debug, Clone)]
pub struct VcdSignalHandle {
    pub id: HashMap<InstanceIndex, vcd::IdCode>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct InstanceIndex {
    instance: usize,
    port: usize,
}

impl InstanceIndex {
    pub fn new(instance: usize, port: usize) -> Self {
        Self { instance, port }
    }
}

// FIXME: This function is the main bottleneck
pub fn write_vcd_signals(writer: &mut vcd::Writer, vi: InstanceIndex, vh: &VcdSignalHandle,
                     signals1: &[Bit], signals2: &[Bit]) -> io::Result<InstanceIndex> {
    let mut vi = vi.clone();

    for s in signals1 {
        let h = vh.id[&vi];
        writer.change_scalar(h, *s)?;
        vi.port += 1;
    }

    for s in signals2 {
        let h = vh.id[&vi];
        writer.change_scalar(h, *s)?;
        vi.port += 1;
    }

    Ok(vi)
}

#[derive(Debug, Copy, Clone)]
pub struct Nand {
    num_inputs: usize,
}

impl Nand {
    pub fn new(num_inputs: usize) -> Nand {
        Nand { num_inputs }
    }
}

impl Component for Nand {
    fn update(&mut self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(self.num_inputs, input.len());
        let mut x = Bit::L;
        for a in input {
            match *a {
                // If any input is 0, the output is 1
                Bit::L => return vec![Bit::H],
                // X NAND L = H, but X NAND H = X
                Bit::X => x = Bit::X,
                Bit::H => {},
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
        "Nand"
    }
    fn box_clone(&self) -> Box<Component> {
        Box::new((*self).clone())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ConstantBit { }

impl ConstantBit {
    pub fn new() -> Self {
        Self { }
    }
}

impl Component for ConstantBit {
    fn update(&mut self, input: &[Bit]) -> Vec<Bit> {
        assert!(input.is_empty());
        vec![Bit::L, Bit::H, Bit::X]
    }
    fn num_inputs(&self) -> usize {
        0
    }
    fn num_outputs(&self) -> usize {
        3
    }
    fn name(&self) -> &str {
        "ConstantBit"
    }
    fn box_clone(&self) -> Box<Component> {
        Box::new((*self).clone())
    }
    fn port_names(&self) -> PortNames {
        PortNames::new(&[], &["o0", "o1", "oX"])
    }
}

#[derive(Debug, Clone)]
pub struct PortNames {
    pub input: Vec<String>,
    pub output: Vec<String>,
}

impl PortNames {
    pub fn new(input: &[&str], output: &[&str]) -> PortNames {
        // TODO: check for duplicate names
        let input = input.iter().map(|x| x.to_string()).collect();
        let output = output.iter().map(|x| x.to_string()).collect();
        PortNames { input, output }
    }
    pub fn new_vec(input: Vec<String>, output: Vec<String>) -> PortNames {
        // TODO: check for duplicate names
        PortNames { input, output }
    }
    pub fn default(num_inputs: usize, num_outputs: usize) -> PortNames {
        let mut input = vec![];
        for i in 0..num_inputs {
            input.push(format!("i{}", i));
        }
        let mut output = vec![];
        for i in 0..num_outputs {
            output.push(format!("o{}", i));
        }

        PortNames { input, output }
    }
}

#[derive(Debug, Clone)]
pub struct Structural {
    pub components: Vec<CompIo>,
    pub info: Rc<CompInfo>,
    pub comp_def: Rc<CompDefinition>,
}

impl Structural {
    pub fn new(components: Vec<CompIo>, num_inputs: usize, num_outputs: usize,
           name: &str, port_names: PortNames) -> Structural {
        // Component 0 must have been created using CompIo::c_zero
        assert_eq!(components[0].input.len(), num_outputs);
        assert_eq!(components[0].output.len(), num_inputs);
        assert_eq!(components[0].connections.len(), num_inputs);
        // Check port_names len is valid
        assert_eq!(port_names.input.len(), num_inputs);
        assert_eq!(port_names.output.len(), num_outputs);
        // TODO: check that all the connections are valid
        let name = name.to_string();
        let PortNames { input, output } = port_names;
        let info = Rc::new(CompInfo::new(name, input, output));
        let mut connections = vec![];
        for c in &components {
            connections.push(c.connections.clone());
        }

        let mut c2 = HashMap::new();
        for (from_comp, vv) in connections.into_iter().enumerate() {
            for (from_port, v) in vv.into_iter().enumerate() {
                let to = v.into_iter().map(|x| ComponentIndex::input(x.comp_id, x.input_id)).collect();
                c2.insert(ComponentIndex::output(from_comp, from_port), to);
            }
        }
        let comp_def = Rc::new(CompDefinition::new_but_only_connections(c2));
        Structural { components, info, comp_def }
    }
    // Create a Structural from one Component
    pub fn new_wrap(component: Box<Component>) -> Structural {
        let port_names = component.port_names();
        let num_inputs = port_names.input.len();
        let num_outputs = port_names.output.len();
        let name = format!("w{}", component.name());
        let mut c_zero = CompIo::c_zero(num_inputs, num_outputs);
        let mut c_one = CompIo::new(component);
        
        for i in 0..num_inputs {
            c_zero.add_connection(i, Index::new(1, i));
        }
        for i in 0..num_outputs {
            c_one.add_connection(i, Index::new(0, i));
        }

        let components = vec![c_zero, c_one];

        Structural::new(components, num_inputs, num_outputs, &name, port_names)
    }
    fn propagate(&mut self, c_id: usize) {
        // TODO: avoid this clone
        let connections = self.components[c_id].connections.clone();
        for (out_id, to) in connections.iter().enumerate() {
            for i in to {
                self.components[i.comp_id]
                    .input[i.input_id] = self.components[c_id].output[out_id];
            }
        }
    }
    fn propagate_input(&mut self, input: &[Bit]) {
        // The input is the output when seen from inside
        self.components[0].output = input.to_vec();
        self.propagate(0);
    }
    fn output(&self) -> Vec<Bit> {
        self.components[0].input.clone()
    }
    fn update_components(&mut self) {
        for c in 1..self.components.len() {
            // Magic pattern matching to make the borrow checker happy
            let CompIo {
                ref mut comp,
                ref input,
                ref mut output,
                connections: _
            } = self.components[c];
            *output = comp.update(input);
        }
    }
    fn propagate_signals(&mut self) {
        for c in 1..self.components.len() {
            self.propagate(c);
        }
    }
}

impl Component for Structural {
    fn update(&mut self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(input.len(), self.num_inputs());
        // Propagate input
        self.propagate_input(input);
        // Update components
        self.update_components();
        // Propagate internal signals
        self.propagate_signals();
        // Return the component output
        self.output()
    }
    fn num_inputs(&self) -> usize {
        self.info.inputs.len()
    }
    fn num_outputs(&self) -> usize {
        self.info.outputs.len()
    }
    fn name(&self) -> &str {
        &self.info.name
    }
    fn write_internal_components(&self, writer: &mut vcd::Writer, j: &mut u64) -> io::Result<VcdSignalHandle> {
        let mut vh = VcdSignalHandle { id: HashMap::new() };
        let write_parent = *j == 0;
        if write_parent {
            let mut vi = InstanceIndex::new(*j as usize, 0);
            let instance_name = format!("{}-{}", self.name(), j);
            writer.add_module(&instance_name)?;
            for i in 0..self.num_inputs() {
                let ref port_name = self.info.inputs[i];
                vh.id.insert(vi, writer.add_wire(1,
                    &format!("{}-{}", instance_name, port_name))?);
                vi.port += 1;
            }
            for i in 0..self.num_outputs() {
                let ref port_name = self.info.outputs[i];
                vh.id.insert(vi, writer.add_wire(1,
                    &format!("{}-{}", instance_name, port_name))?);
                vi.port += 1;
            }

            *j += 1;
        }

        for c in self.components.iter().skip(1).filter(|&c| VCD_SHOW_NAND || (c.comp.name() != "NAND")) {
            let mut vi = InstanceIndex::new(*j as usize, 0);
            let instance_name = format!("{}-{}", c.comp.name(), j);
            writer.add_module(&instance_name)?;
            let port_names = c.comp.port_names();
            for i in 0..c.comp.num_inputs() {
                let ref port_name = port_names.input[i];
                vh.id.insert(vi, writer.add_wire(1,
                    &format!("{}-{}", instance_name, port_name))?);
                vi.port += 1;
            }
            for i in 0..c.comp.num_outputs() {
                let ref port_name =port_names.output[i];
                vh.id.insert(vi, writer.add_wire(1,
                    &format!("{}-{}", instance_name, port_name))?);
                vi.port += 1;
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

    fn write_internal_signals(&self, writer: &mut vcd::Writer, j: &mut u64, vh: &VcdSignalHandle) -> io::Result<()> {
        let write_parent = *j == 0;

        if write_parent {
            // TODO: create a less error prone helper method
            let ref inputs = self.components[0].output;
            let ref outputs = self.components[0].input;
            let vi = InstanceIndex::new(*j as usize, 0);
            write_vcd_signals(writer, vi, vh, inputs, outputs)?;
            *j += 1;
        }

        for c in self.components.iter().skip(1).filter(|&c| VCD_SHOW_NAND || (c.comp.name() != "NAND")) {
            let ref inputs = c.input;
            let ref outputs = c.output;
            let vi = InstanceIndex::new(*j as usize, 0);
            write_vcd_signals(writer, vi, vh, inputs, outputs)?;
            *j += 1;

            c.comp.write_internal_signals(writer, j, vh)?;
        }

        Ok(())
    }
    fn port_names(&self) -> PortNames {
        PortNames::new_vec(self.info.inputs.clone(), self.info.outputs.clone())
    }
    fn clone_as_structural(&self) -> Option<Structural> {
        Some(self.clone())
    }
    fn box_clone(&self) -> Box<Component> {
        Box::new((*self).clone())
    }
}

#[derive(Debug, Clone)]
pub struct CompIo {
    pub comp: Box<Component>,
    input: Vec<Bit>,
    output: Vec<Bit>,
    pub connections: Vec<Vec<Index>>,
}

impl CompIo {
    pub fn new(comp: Box<Component>) -> CompIo {
        let input = vec![Bit::X; comp.num_inputs()];
        let output = vec![Bit::X; comp.num_outputs()];
        let connections = vec![vec![]; comp.num_inputs()];
        CompIo {
            comp,
            input,
            output,
            connections,
        }
    }
    pub fn c_zero(num_inputs: usize, num_outputs: usize) -> CompIo {
        let comp = Box::new(Nand::new(0));
        let input = vec![Bit::X; num_outputs];
        let output = vec![Bit::X; num_inputs];
        let connections = vec![vec![]; num_inputs];
        CompIo {
            comp,
            input,
            output,
            connections,
        }
    }
    pub fn add_connection(&mut self, output_id: usize, to: Index) {
        self.connections[output_id].push(to);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Index {
    pub comp_id: usize,
    pub input_id: usize
}

impl Index {
    pub fn new(comp_id: usize, input_id: usize) -> Index {
        Index { comp_id, input_id }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ComponentIndex {
    pub c_id: usize,
    pub port_id: usize,
    pub direction: Direction,
}

impl ComponentIndex {
    pub fn input(c_id: usize, port_id: usize) -> Self {
        Self { c_id, port_id, direction: Direction::Input }
    }
    pub fn output(c_id: usize, port_id: usize) -> Self {
        Self { c_id, port_id, direction: Direction::Output }
    }
    pub fn is_output(&self) -> bool {
        self.direction == Direction::Output
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[serde(rename="input")]
    Input,
    #[serde(rename="output")]
    Output,
}
