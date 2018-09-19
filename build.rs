extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .process_file("src/comphdl1.lalrpop")
        .unwrap();
    println!("cargo:rerun-if-changed=src/comphdl1.lalrpop");
}
