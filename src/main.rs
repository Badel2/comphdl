#![cfg_attr(feature = "stdweb", feature(proc_macro))]
#![recursion_limit = "256"]

#[cfg(feature = "stdweb")]
#[macro_use]
extern crate stdweb;

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

#[cfg(feature = "stdweb")]
pub mod js_gui;
#[cfg(feature = "stdweb")]
pub use js_gui::*;

use bit::RepInputIterator;
use component::Component;
use simulation::run_simulation;
use std::io::{BufReader, Read, Write};
use std::fs::File;

fn yosys_netlist(c: &Component) {
    // We can only generate netlists from structural, not from component
    let c = c.clone_as_structural().unwrap();
    let s = emit_json::from_structural(&c).unwrap();
    println!("{}", s);
}

pub fn parse_file(filename: &str, top: &str) {
    let file = File::open(filename).expect("Unable to open file");
    let mut buf_reader = BufReader::new(file);
    let mut bs = String::new();
    buf_reader.read_to_string(&mut bs).unwrap();

    let cf = parser::parse_str(&bs).unwrap();
    let mux = cf.create_named(top).unwrap();
    println!("{:#?}", mux);

    let mut gate = mux;

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


// Do not start automatically when loaded from js
#[cfg(feature = "stdweb")]
fn main(){}

#[cfg(not(feature = "stdweb"))]
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
