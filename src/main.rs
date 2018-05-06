extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate vcd;

mod emit_json;
mod parser;
mod bit;
mod component;
pub mod comphdl1;

use vcd::{ Value, TimescaleUnit, SimulationCommand };
use bit::{Bit, RepInputIterator};
use component::Component;
use std::io;
use std::io::Write;
use std::fs::File;

pub fn run_simulation(w: &mut io::Write,
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

fn yosys_netlist(c: &Component) {
    // We can only generate netlists from structural, not from component
    let c = c.clone_as_structural().unwrap();
    let s = emit_json::from_structural(&c).unwrap();
    println!("{}", s);
}

fn parse_file(filename: &str, top: &str) {
    // Create gate
    let mut gate = parser::parse_file(filename, top);

    // Run simulation
    let mut buf = Vec::with_capacity(20_000_000);
    let mut input = RepInputIterator::new(10, 50);
    run_simulation(&mut buf, &mut *gate, &mut input, 4000).unwrap();

    // Write simulation to foo.vcd
    let mut file = File::create("foo.vcd").expect("Unable to create file");
    file.write_all(&buf).expect("Error writing vcd");

    // Print netlist JSON
    yosys_netlist(&*gate);
}

fn main(){
    // Usage: cargo run (for default arguments)
    //        cargo run -- test.txt Buf123 (filename, component name)
    use std::env;
    let mut args = env::args();
    let _program_name = args.next().unwrap();
    let filename = args.next().unwrap_or(format!("test.txt"));
    let top = args.next().unwrap_or(format!("Demux_1_4"));
    parse_file(&filename, &top);
}
