# comphdl

A digital logic simulator with its own programming language.

Early demo available at:

https://badel2.github.io/comphdl/demo/v09/

Usage:

Load an example, enter a component name, click "RUN". This will generate a
visual representation of the component, you can click on the inputs to toggle
them.

This is a work in progress, you can follow it on:

https://badel2.github.io/2018/02/08/comphdl-01.html

# Dependencies

The Rust dependencies are listed in `Cargo.toml` file, but the `js` libraries
are listed here, as we do not have a `package.json` file yet:

* [netlistsvg](https://github.com/nturley/netlistsvg): Draws an SVG schematic from a JSON netlist
* [ACE](https://ace.c9.io/): A code editor for the web
* [stats.js](https://github.com/mrdoob/stats.js): A simple FPS widget
* [WaveDrom](https://wavedrom.com/): Digital timing diagram rendering engine
* [xtermjs](https://xtermjs.org/): A terminal for the web


