extern crate comphdl;

#[macro_use]
extern crate log;
extern crate env_logger;

use comphdl::bit::RepInputIterator;
use comphdl::component::Component;
use comphdl::simulation::run_simulation;
use comphdl::{emit_json, parser};
use std::io::{BufReader, Read, Write};
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

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

    let mut cf = parser::parse_str(&bs).unwrap();
    // If file stdin.txt exists, read input from there instead of stdin
    if let Ok(stdin_bufread) = File::open("stdin.txt") {
        info!("Reading input from stdin.txt");
        cf.set_stdin_bufread(Rc::new(RefCell::new(BufReader::new(stdin_bufread))));
    }
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

fn main(){
    env_logger::init();
    // Usage: cargo run (for default arguments)
    //        cargo run -- test.txt Buf123 (filename, component name)
    use std::env;
    let mut args = env::args();
    let _program_name = args.next().unwrap();
    let filename = args.next().unwrap_or(format!("test.txt"));
    let top = args.next().unwrap_or(format!("Demux_1_4"));
    parse_file(&filename, &top);
}

