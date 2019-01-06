use vcd::{ self, Value, TimescaleUnit, SimulationCommand };
use crate::bit::Bit;
use crate::component::Component;
use std::io;

pub fn run_simulation(w: &mut io::Write,
                  c: &mut Component,
                  inputs: &mut Iterator<Item=Vec<Bit>>,
                  ticks: usize) -> io::Result<()> {
    let mut writer = vcd::Writer::new(w);

    {
        let a = c.clone_as_structural();
        info!("{:#?}", a);
    }

    // Write the header
    writer.timescale(1, TimescaleUnit::NS)?; // 1 tick = 1 ns

    let vh = c.write_internal_components(&mut writer, &mut 0)?;
    writer.add_module(&format!("clk"))?;
    let clk = writer.add_wire(1, "clk")?;
    writer.upscope()?;

    writer.enddefinitions()?;

    // Write the initial values
    writer.begin(SimulationCommand::Dumpvars)?;
    writer.change_scalar(clk, Bit::L)?;
    // Initialize everything to X
    for h in vh.id.values() {
        writer.change_scalar(*h, Bit::X)?;
    }
    writer.end()?;

    let num_inputs = c.num_inputs();
    // Write the data values
    let mut clk_on = true;
    let mut t = 0;
    for current_input in inputs.take(ticks) {
        writer.timestamp(t)?;
        let input_slice = current_input.len() - num_inputs;
        let _outputs = c.update(&current_input[input_slice..input_slice + num_inputs]);
        //println!("{:?}", outputs);
        c.write_internal_signals(&mut writer, &mut 0, &vh)?;
        writer.change_scalar(clk, if clk_on { Value::V1 } else { Value::V0 })?;
        clk_on = !clk_on;
        t += 1;
    }
    writer.timestamp(t)?;

    Ok(())
}

