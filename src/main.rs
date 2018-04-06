extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate vcd;

mod emit_json;

use std::io;
use std::io::Write;
use std::fs::File;
use vcd::{ Value, TimescaleUnit, SimulationCommand };
use std::collections::HashMap;

static VCD_SHOW_NAND: bool = true;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Bit {
    L, // Low, false, 0
    H, // High, true, 1
    X, // Undefined
}

impl From<Bit> for vcd::Value {
    fn from(x: Bit) -> Self {
        match x {
            Bit::L => Value::V0,
            Bit::H => Value::V1,
            Bit::X => Value::X,
        }
    }
}

// Returns all the n-bit combinations in order, loops infinitely
struct InfiniteInputIterator {
    current: Vec<Bit>,
}

impl InfiniteInputIterator {
    fn new(n: usize) -> Self {
        Self { current: vec![Bit::L; n] }
    }
}

impl Iterator for InfiniteInputIterator {
    type Item = Vec<Bit>;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.current.clone();
        self.current = next_bit_combination(&a);

        Some(a)
    }
}

#[derive(Debug, Clone)]
struct RepInputIterator {
    current: Vec<Bit>,
    count: u32,
    max_count: u32,
}

impl RepInputIterator {
    fn new(n: usize, rep: u32) -> Self {
        Self {
            current: vec![Bit::L; n],
            count: rep,
            max_count: rep,
        }
    }
}

impl Iterator for RepInputIterator {
    type Item = Vec<Bit>;

    fn next(&mut self) -> Option<Vec<Bit>> {
        let a = self.current.clone();
        self.count -= 1;
        if self.count == 0 {
            self.current = next_bit_combination(&a);
            self.count = self.max_count;
        }

        Some(a)
    }
}

fn all_n_bit_combinations(n: usize) -> Vec<Vec<Bit>> {
    let mut res = vec![];
    let mut a = vec![Bit::L; n];
    res.push(a.clone());
    loop {
        a = next_bit_combination(&a);
        res.push(a.clone());
        if a.iter().all(|&x| x == Bit::H) {
            break;
        }
    }

    res
}

fn next_bit_combination(x: &[Bit]) -> Vec<Bit> {
    let mut y = x.to_vec();
    let mut i = x.len();
    let mut carry = true;
    while carry == true && i > 0 {
        i -= 1;
        match x[i] {
            Bit::L => { carry = false; y[i] = Bit::H; },
            Bit::H => { carry = true; y[i] = Bit::L; },
            // Incrementing X would make all the higher bits X, we don't want
            // that, instead we keep it as X and increment the next bit
            Bit::X => { carry = true; },
        }
    }

    y
}

trait Component: std::fmt::Debug {
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
        None
    }
    fn box_clone(&self) -> Box<Component>;
}

impl Clone for Box<Component> {
    fn clone(&self) -> Box<Component> {
        self.box_clone()
    }
}

#[derive(Debug, Clone)]
struct VcdSignalHandle {
    id: HashMap<InstanceIndex, vcd::IdCode>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct InstanceIndex {
    instance: usize,
    port: usize,
}

impl InstanceIndex {
    fn new(instance: usize, port: usize) -> Self {
        Self { instance, port }
    }
}

// FIXME: This function is the main bottleneck
fn write_vcd_signals(writer: &mut vcd::Writer, vi: InstanceIndex, vh: &VcdSignalHandle,
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

fn run_simulation(w: &mut io::Write,
                  c: &mut Component,
                  inputs: &mut Iterator<Item=Vec<Bit>>,
                  ticks: usize) -> io::Result<()> {
    let mut writer = vcd::Writer::new(w);

    let a = c.clone_as_structural();
    println!("{:#?}", a);

    // Write the header
    writer.timescale(1, TimescaleUnit::NS)?; // 1 tick = 1 ns

    let vh = c.write_internal_components(&mut writer, &mut 0)?;
    writer.add_module(&format!("clk"))?;
    let clk = writer.add_wire(1, "clk")?;
    writer.upscope()?;

    writer.enddefinitions()?;

    // Write the initial values
    writer.begin(SimulationCommand::Dumpvars)?;
    writer.change_scalar(clk, Bit::L)?;
    // Initialize everything to X
    for h in vh.id.values() {
        writer.change_scalar(*h, Bit::X)?;
    }
    writer.end()?;

    let num_inputs = c.num_inputs();
    // Write the data values
    let mut clk_on = true;
    let mut t = 0;
    for current_input in inputs.take(ticks) {
        writer.timestamp(t)?;
        let input_slice = current_input.len() - num_inputs;
        let _outputs = c.update(&current_input[input_slice..input_slice + num_inputs]);
        //println!("{:?}", outputs);
        c.write_internal_signals(&mut writer, &mut 0, &vh)?;
        writer.change_scalar(clk, if clk_on { Value::V1 } else { Value::V0 })?;
        clk_on = !clk_on;
        t += 1;
    }
    writer.timestamp(t)?;

    Ok(())
}

#[derive(Debug, Copy, Clone)]
struct Nand {
    num_inputs: usize,
}

impl Nand {
    fn new(num_inputs: usize) -> Nand {
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
        "NAND"
    }
    fn box_clone(&self) -> Box<Component> {
        Box::new((*self).clone())
    }
}

#[derive(Debug, Copy, Clone)]
struct ConstantBit { }

impl ConstantBit {
    fn new() -> Self {
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
        "GND-VCC"
    }
    fn box_clone(&self) -> Box<Component> {
        Box::new((*self).clone())
    }
    fn port_names(&self) -> PortNames {
        PortNames::new(&[], &["o0", "o1", "oX"])
    }
}

#[derive(Debug, Copy, Clone)]
struct Or2 {
    nand_a: Nand,
    nand_b: Nand,
    nand_c: Nand,
}

impl Or2 {
    fn new() -> Or2 {
        Or2 {
            nand_a: Nand::new(1),
            nand_b: Nand::new(1),
            nand_c: Nand::new(2),
        }
    }
}

impl Component for Or2 {
    fn update(&mut self, input: &[Bit]) -> Vec<Bit> {
        assert_eq!(input.len(), 2);
        let a = input[0];
        let b = input[1];
        let not_a = self.nand_a.update(&[a])[0];
        let not_b = self.nand_b.update(&[b])[0];
        // not_a nand not_b == not (not_a or not_b) == a or b
        self.nand_c.update(&[not_a, not_b])
    }
    fn num_inputs(&self) -> usize {
        2
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn name(&self) -> &str {
        "OR"
    }
    fn box_clone(&self) -> Box<Component> {
        Box::new((*self).clone())
    }
}

#[derive(Debug, Clone)]
struct PortNames {
    input: Vec<String>,
    output: Vec<String>,
}

impl PortNames {
    fn new(input: &[&str], output: &[&str]) -> PortNames {
        // TODO: check for duplicate names
        let input = input.iter().map(|x| x.to_string()).collect();
        let output = output.iter().map(|x| x.to_string()).collect();
        PortNames { input, output }
    }
    fn default(num_inputs: usize, num_outputs: usize) -> PortNames {
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
    components: Vec<CompIo>,
    num_inputs: usize,
    num_outputs: usize,
    name: String,
    port_names: PortNames,
}

impl Structural {
    fn new(components: Vec<CompIo>, num_inputs: usize, num_outputs: usize,
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
        Structural { components, num_inputs, num_outputs, name, port_names }
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
        self.num_inputs
    }
    fn num_outputs(&self) -> usize {
        self.num_outputs
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn write_internal_components(&self, writer: &mut vcd::Writer, j: &mut u64) -> io::Result<VcdSignalHandle> {
        let mut vh = VcdSignalHandle { id: HashMap::new() };
        let write_parent = *j == 0;
        if write_parent {
            let mut vi = InstanceIndex::new(*j as usize, 0);
            let instance_name = format!("{}-{}", self.name(), j);
            writer.add_module(&instance_name)?;
            for i in 0..self.num_inputs {
                let ref port_name = self.port_names.input[i];
                vh.id.insert(vi, writer.add_wire(1,
                    &format!("{}-{}", instance_name, port_name))?);
                vi.port += 1;
            }
            for i in 0..self.num_outputs {
                let ref port_name = self.port_names.output[i];
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
        self.port_names.clone()
    }
    fn clone_as_structural(&self) -> Option<Structural> {
        Some(self.clone())
    }
    fn box_clone(&self) -> Box<Component> {
        Box::new((*self).clone())
    }
}

#[derive(Debug, Clone)]
struct CompIo {
    comp: Box<Component>,
    input: Vec<Bit>,
    output: Vec<Bit>,
    connections: Vec<Vec<Index>>,
}

impl CompIo {
    fn new(comp: Box<Component>) -> CompIo {
        let input = vec![Bit::X; comp.num_inputs()];
        let output = vec![Bit::X; comp.num_outputs()];
        let connections = vec![vec![]; comp.num_outputs()];
        CompIo {
            comp,
            input,
            output,
            connections,
        }
    }
    fn c_zero(num_inputs: usize, num_outputs: usize) -> CompIo {
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
    fn add_connection(&mut self, output_id: usize, to: Index) {
        self.connections[output_id].push(to);
    }
}

#[derive(Debug, Copy, Clone)]
struct Index {
    comp_id: usize,
    input_id: usize
}

impl Index {
    fn new(comp_id: usize, input_id: usize) -> Index {
        Index { comp_id, input_id }
    }
}

fn nand_short() {
    let mut c = vec![];
    let mut c_zero = CompIo::c_zero(1, 1); // c_id: 0
    let mut nand_a = CompIo::new(Box::new(Nand::new(2))); // c_id: 1

    c_zero.add_connection(0, Index::new(1, 0)); // input 0 -> nand_a
    nand_a.add_connection(0, Index::new(1, 1)); // nand_a -> nand_a
    nand_a.add_connection(0, Index::new(0, 0)); // nand_a -> out
    
    c.push(c_zero);
    c.push(nand_a);
    let pn = PortNames::default(1, 1);
    let mut nand_short = Structural::new(c, 1, 1, "NAND short", pn);
    truth_table(&mut nand_short);
}

fn rs_latch() {
    let mut c = vec![];
    let mut c_zero = CompIo::c_zero(2, 2); // c_id: 0
    let mut nand_a = CompIo::new(Box::new(Nand::new(2))); // c_id: 1
    let mut nand_b = CompIo::new(Box::new(Nand::new(2))); // c_id: 2

    c_zero.add_connection(0, Index::new(1, 0)); // input 0 -> nand_a
    c_zero.add_connection(1, Index::new(2, 0)); // input 1 -> nand_b
    nand_a.add_connection(0, Index::new(2, 1)); // nand_a -> nand_b
    nand_b.add_connection(0, Index::new(1, 1)); // nand_b -> nand_a
    nand_a.add_connection(0, Index::new(0, 0)); // nand_a -> out
    nand_b.add_connection(0, Index::new(0, 1)); // nand_b -> out
    
    c.push(c_zero);
    c.push(nand_a);
    c.push(nand_b);
    let pn = PortNames::new(&["R", "S"], &["Q", "Q'"]);
    let mut rs_latch = Structural::new(c, 2, 2, "RS Latch", pn);
    truth_table(&mut rs_latch);
}

fn boxed_or_gate() -> Box<Component> {
    let mut c = vec![];
    let mut c_zero = CompIo::c_zero(2, 1); // c_id: 0
    let mut nand_a = CompIo::new(Box::new(Nand::new(1))); // c_id: 1
    let mut nand_b = CompIo::new(Box::new(Nand::new(1))); // c_id: 2
    let mut nand_c = CompIo::new(Box::new(Nand::new(2))); // c_id: 3

    c_zero.add_connection(0, Index::new(1, 0)); // input 0 -> nand_a
    c_zero.add_connection(1, Index::new(2, 0)); // input 1 -> nand_b
    nand_a.add_connection(0, Index::new(3, 0)); // nand_a -> nand_c
    nand_b.add_connection(0, Index::new(3, 1)); // nand_b -> nand_c
    nand_c.add_connection(0, Index::new(0, 0)); // output of nand_c == output of or

    c.push(c_zero);
    c.push(nand_a);
    c.push(nand_b);
    c.push(nand_c);
    
    let pn = PortNames::default(2, 1);
    Box::new(Structural::new(c, 2, 1, "OR2", pn))
}

fn or_gate() {
    let mut or_gate = boxed_or_gate();
    truth_table(&mut *or_gate);
}

fn old_or_gate() {
    let mut or_gate = Or2::new();
    truth_table(&mut or_gate);
}

fn boxed_triple_or() -> Box<Component> {
    let mut c = vec![];
    let mut c_zero = CompIo::c_zero(1, 1); // c_id: 0
    let mut or_a = CompIo::new(boxed_or_gate()); // c_id: 1
    let mut or_b = CompIo::new(boxed_or_gate()); // c_id: 2
    let mut or_c = CompIo::new(boxed_or_gate()); // c_id: 3

    c_zero.add_connection(0, Index::new(1, 0)); // input 0 -> or_a
    c_zero.add_connection(0, Index::new(1, 1)); // input 0 -> or_a
    or_a.add_connection(0, Index::new(2, 0)); // or_a -> or_b
    or_a.add_connection(0, Index::new(2, 1)); // or_a -> or_b
    or_b.add_connection(0, Index::new(3, 0)); // or_b -> or_c
    or_b.add_connection(0, Index::new(3, 1)); // or_b -> or_c
    or_c.add_connection(0, Index::new(0, 0)); // output

    c.push(c_zero);
    c.push(or_a);
    c.push(or_b);
    c.push(or_c);
    let pn = PortNames::default(1, 1);
    let or_gate = Structural::new(c, 1, 1, "OR-OR-OR", pn);
    let s = emit_json::from_structural(&or_gate).unwrap();
    println!("{}", s);

    Box::new(or_gate)
}

fn triple_or() {
    let mut c = vec![];
    let mut c_zero = CompIo::c_zero(1, 1); // c_id: 0
    let mut or_a = CompIo::new(boxed_or_gate()); // c_id: 1
    let mut or_b = CompIo::new(boxed_or_gate()); // c_id: 2
    let mut or_c = CompIo::new(boxed_or_gate()); // c_id: 3

    c_zero.add_connection(0, Index::new(1, 0)); // input 0 -> or_a
    c_zero.add_connection(0, Index::new(1, 1)); // input 0 -> or_a
    or_a.add_connection(0, Index::new(2, 0)); // or_a -> or_b
    or_a.add_connection(0, Index::new(2, 1)); // or_a -> or_b
    or_b.add_connection(0, Index::new(3, 0)); // or_b -> or_c
    or_b.add_connection(0, Index::new(3, 1)); // or_b -> or_c
    or_c.add_connection(0, Index::new(0, 0)); // output

    c.push(c_zero);
    c.push(or_a);
    c.push(or_b);
    c.push(or_c);
    let pn = PortNames::default(1, 1);
    let mut or_gate = Structural::new(c, 1, 1, "OR-OR-OR", pn);
    truth_table(&mut or_gate);
}

fn truth_table(or_gate: &mut Component) {
    let n = or_gate.num_inputs();
    println!("{} truth table:", or_gate.name());
    let mut i = vec![Bit::L, Bit::L];
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    //println!("{:#?}", or_gate);
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    i = vec![Bit::L, Bit::H];
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    i = vec![Bit::H, Bit::L];
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    i = vec![Bit::H, Bit::H];
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
    println!("{:?} = {:?}", &i[0..n], or_gate.update(&i[0..n]));
}

fn mux_4_1() -> Structural {
    let mut c = vec![];
    // a1, a0, A, B, C, D
    let mut c_zero = CompIo::c_zero(6, 1); // c_id: 0
    let mut not_a1 = CompIo::new(Box::new(Nand::new(1))); // c_id: 1
    let mut not_a0 = CompIo::new(Box::new(Nand::new(1))); // c_id: 2
    let mut sel00 = CompIo::new(Box::new(Nand::new(3))); // c_id: 3
    let mut sel01 = CompIo::new(Box::new(Nand::new(3))); // c_id: 4
    let mut sel10 = CompIo::new(Box::new(Nand::new(3))); // c_id: 5
    let mut sel11 = CompIo::new(Box::new(Nand::new(3))); // c_id: 6
    let mut selor = CompIo::new(Box::new(Nand::new(4))); // c_id: 7

    c_zero.add_connection(0, Index::new(1, 0)); // a1 -> not_a1
    c_zero.add_connection(1, Index::new(2, 0)); // a0 -> not_a0
    c_zero.add_connection(0, Index::new(5, 0)); // a1 -> sel10
    c_zero.add_connection(0, Index::new(6, 0)); // a1 -> sel11
    c_zero.add_connection(1, Index::new(4, 0)); // a0 -> sel01
    c_zero.add_connection(1, Index::new(6, 1)); // a0 -> sel11
    not_a1.add_connection(0, Index::new(3, 0)); // not_a1 -> sel00
    not_a1.add_connection(0, Index::new(4, 1)); // not_a1 -> sel01
    not_a0.add_connection(0, Index::new(3, 1)); // not_a0 -> sel00
    not_a0.add_connection(0, Index::new(5, 1)); // not_a0 -> sel10
    c_zero.add_connection(2, Index::new(3, 2)); // A -> sel00
    c_zero.add_connection(3, Index::new(4, 2)); // B -> sel01
    c_zero.add_connection(4, Index::new(5, 2)); // C -> sel10
    c_zero.add_connection(5, Index::new(6, 2)); // D -> sel11
    sel00.add_connection(0, Index::new(7, 0)); //
    sel01.add_connection(0, Index::new(7, 1)); //
    sel10.add_connection(0, Index::new(7, 2)); //
    sel11.add_connection(0, Index::new(7, 3)); //
    selor.add_connection(0, Index::new(0, 0)); // output

    c.push(c_zero);
    c.push(not_a1);
    c.push(not_a0);
    c.push(sel00);
    c.push(sel01);
    c.push(sel10);
    c.push(sel11);
    c.push(selor);
    let pn = PortNames::new(&["S1", "S0", "A", "B", "C", "D"], &["Y"]);
    let mut or_gate = Structural::new(c, 6, 1, "MUX-4-TO-1", pn);

    or_gate
}

fn mux_xor() -> Structural {
    // a1, a0
    let mut c_zero = CompIo::c_zero(2, 1); // c_id: 0
    let mut mux41 = CompIo::new(Box::new(mux_4_1())); // c_id: 1
    let mut gndvcc = CompIo::new(Box::new(ConstantBit::new())); // c_id: 2

    c_zero.add_connection(0, Index::new(1, 0)); // s1 -> mux s1
    c_zero.add_connection(1, Index::new(1, 1)); // s0 -> mux s0
    gndvcc.add_connection(0, Index::new(1, 2)); // 0 -> sel00
    gndvcc.add_connection(1, Index::new(1, 3)); // 1 -> sel01
    gndvcc.add_connection(1, Index::new(1, 4)); // 1 -> sel10
    gndvcc.add_connection(0, Index::new(1, 5)); // 0 -> sel11
    mux41.add_connection(0, Index::new(0, 0)); // mux y -> y

    let c = vec![c_zero, mux41, gndvcc];
    let pn = PortNames::new(&["S1", "S0"], &["Y"]);
    let mut mux_xor = Structural::new(c, 2, 1, "MUX-XOR", pn);

    mux_xor
}

fn buffer() -> Structural {
    let mut c_zero = CompIo::c_zero(1, 1);
    c_zero.add_connection(0, Index::new(0, 0));
    let c = vec![c_zero];
    let pn = PortNames::new(&["D"], &["Q"]);
    let mut d = Structural::new(c, 1, 1, "D-Latch", pn);

    d
}

fn glitchless_xor() -> Structural {
    // a1, a0, A, B, C, D
    let mut c_zero = CompIo::c_zero(2, 1); // c_id: 0
    let mut not_a1 = CompIo::new(Box::new(Nand::new(1))); // c_id: 1
    let mut not_a0 = CompIo::new(Box::new(Nand::new(1))); // c_id: 2
    let mut sel01 = CompIo::new(Box::new(Nand::new(2))); // c_id: 3
    let mut sel10 = CompIo::new(Box::new(Nand::new(2))); // c_id: 4
    let mut selor = CompIo::new(Box::new(Nand::new(2))); // c_id: 5
    let mut buf0 = CompIo::new(Box::new(buffer())); // c_id: 6
    let mut buf1 = CompIo::new(Box::new(buffer())); // c_id: 7

    c_zero.add_connection(0, Index::new(1, 0)); // a1 -> not_a1
    c_zero.add_connection(1, Index::new(2, 0)); // a0 -> not_a0
    c_zero.add_connection(0, Index::new(7, 0)); // a1 -> sel10
    c_zero.add_connection(1, Index::new(6, 0)); // a0 -> sel01
    not_a1.add_connection(0, Index::new(3, 1)); // not_a1 -> sel01
    not_a0.add_connection(0, Index::new(4, 1)); // not_a0 -> sel10
    sel01.add_connection(0, Index::new(5, 0)); //
    sel10.add_connection(0, Index::new(5, 1)); //
    selor.add_connection(0, Index::new(0, 0)); // output
    buf1.add_connection(0, Index::new(4, 0)); // a1 -> sel10
    buf0.add_connection(0, Index::new(3, 0)); // a0 -> sel01

    let mut c = vec![];
    c.push(c_zero);
    c.push(not_a1);
    c.push(not_a0);
    c.push(sel01);
    c.push(sel10);
    c.push(selor);
    c.push(buf0);
    c.push(buf1);
    let pn = PortNames::new(&["S1", "S0"], &["Y"]);
    let mut or_gate = Structural::new(c, 2, 1, "XOR2", pn);

    or_gate
}

fn write_vcd() {
    let mut buf = Vec::with_capacity(20_000_000);
    let mut input = RepInputIterator::new(2, 50);
    /*
    let mut input = vec![vec![Bit::L, Bit::L, Bit::L, Bit::H, Bit::H, Bit::L]; 5];
    input.push(vec![Bit::H, Bit::H, Bit::L, Bit::H, Bit::H, Bit::L]);
    input.push(vec![Bit::H, Bit::H, Bit::L, Bit::H, Bit::H, Bit::L]);
    input.push(vec![Bit::H, Bit::H, Bit::L, Bit::H, Bit::H, Bit::L]);
    input.push(vec![Bit::H, Bit::H, Bit::L, Bit::H, Bit::H, Bit::L]);
    input.push(vec![Bit::H, Bit::H, Bit::L, Bit::H, Bit::H, Bit::L]);
    */
    //let mut input = std::iter::repeat(vec![Bit::L, Bit::L, Bit::L, Bit::H, Bit::H, Bit::L]).take(5).chain(std::iter::repeat(vec![Bit::H, Bit::H, Bit::L, Bit::H, Bit::H, Bit::L]).take(5)).cycle();
    println!("Cool iterator: {:#?}", input);
    let mut or_gate = Box::new(glitchless_xor());
    run_simulation(&mut buf, &mut *or_gate, &mut input, 400).unwrap();

    let mut file = File::create("foo.vcd").expect("Unable to create file");
    file.write_all(&buf).expect("Error writing vcd");
}

fn yosys_netlist() {
    let mut c = glitchless_xor();
    let s = emit_json::from_structural(&c).unwrap();
    println!("{}", s);
}

fn main(){
    old_or_gate();
    or_gate();
    rs_latch();
    nand_short();
    triple_or();
    write_vcd();
    yosys_netlist();
}
