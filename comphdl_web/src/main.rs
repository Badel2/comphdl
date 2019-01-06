#![recursion_limit = "256"]



#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[macro_use]
extern crate log;



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
