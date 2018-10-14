#![recursion_limit = "256"]

extern crate comphdl;

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ansi_term;

#[cfg(target_arch = "wasm32")]
mod stdweb_logger;
#[cfg(target_arch = "wasm32")]
pub mod js_gui;

#[cfg(target_arch = "wasm32")]
fn main() {
    stdweb_logger::Logger::init_with_level(::log::LevelFilter::Debug);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
}
