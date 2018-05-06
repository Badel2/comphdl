extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate vcd;

mod emit_json;
mod parser;
mod bit;
mod component;
mod simulation;
pub mod comphdl1;

use bit::RepInputIterator;
use component::Component;
use simulation::run_simulation;
use std::io::Write;
use std::fs::File;

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
