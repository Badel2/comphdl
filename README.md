# comphdl

A digital logic simulator with its own programming language.

Early demo available at:

https://badel2.github.io/comphdl/demo/v10/

[Video demo - SR Latch](https://i.imgur.com/RWHww0e.mp4)

Usage:

Load an example, enter a component name, click "RUN". This will generate a
visual representation of the component, you can click on the inputs to toggle
them.

This is a work in progress, you can follow it on:

https://badel2.github.io/2018/02/08/comphdl-01.html

# Dependencies

The Rust dependencies are listed in `comphdl_core/Cargo.toml` and
`comphdl_web/Cargo.toml`, and the JavaScript dependencies are in
`package.json`. The most relevant ones are:

Rust:
* [LALRPOP](https://github.com/lalrpop/lalrpop): LR(1) parser generator for Rust 
* [stdweb](https://github.com/koute/stdweb): A standard library for the client-side Web

JavaScript:
* [netlistsvg](https://github.com/nturley/netlistsvg): Draws an SVG schematic from a JSON netlist, using [ELK](https://www.npmjs.com/package/elkjs)
* [ACE](https://ace.c9.io/): A code editor for the web
* [WaveDrom](https://wavedrom.com/): Digital timing diagram rendering engine
* [xtermjs](https://xtermjs.org/): A terminal for the web


