extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate vcd;
#[macro_use]
extern crate lalrpop_util;
#[macro_use]
extern crate log;

pub mod emit_json;
pub mod wave_json;
pub mod parser;
pub mod bit;
pub mod component;
pub mod simulation;
lalrpop_mod!(pub comphdl1);

