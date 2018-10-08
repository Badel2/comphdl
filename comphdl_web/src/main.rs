#![recursion_limit = "256"]

extern crate comphdl;

#[macro_use]
extern crate stdweb;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ansi_term;

mod stdweb_logger;
pub mod js_gui;

fn main() {
    stdweb_logger::Logger::init_with_level(::log::LevelFilter::Debug);
}
