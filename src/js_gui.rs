use std;
use parser;
use emit_json;
use component::ComponentIndex;
use bit::Bit;
use stdweb::js_export;
use stdweb::web::html_element::{InputElement, TextAreaElement};
use stdweb::web::document;
use stdweb::web::IParentNode;
use stdweb::unstable::TryInto;

fn get_element_by_id_value(id: &str) -> String {
    let checked_raw = js! {
        var t = document.getElementById(@{id});
        if(t == null){ return null; }
        return t.value;
    };
    if checked_raw.is_null() {
        console!(error, format!("get_element_by_id_value is null: {}", id));
        return format!("");
    }
    match checked_raw.into_string() {
        Some(s) => {
            s
        }
        None => {
            console!(error, format!("get_element_by_id_value: {}", id));

            format!("")
        }
    }
}

#[js_export]
pub fn run_js_gui() {
    // TODO: check if already running
    let definition = get_element_by_id_value("comphdl_definition");
    let top = get_element_by_id_value("top_name");

    console!(log, "Ok1");
    console!(log, format!("{:?}\n{:?}", definition, top));
    let mut cf = parser::parse_str(&definition);
    let mut c = cf.create_named(&top);

    // Borrow the component as a structural to generate the netlist
    let (s, yosys_addr) = {
        let cs = c.as_structural().unwrap();
        let s = emit_json::from_structural(&cs).unwrap();
        let yosys_addr = emit_json::yosys_addr_map(&cs);

        (s, yosys_addr)
    };

    let comphdl_json: TextAreaElement = document().query_selector( "#comphdl_json" ).unwrap().unwrap().try_into().unwrap();
    comphdl_json.set_value(&s);

    let num_inputs = c.num_inputs();
    let num_outputs = c.num_outputs();
    let pn = c.port_names();
    
    // Create checkboxes for input and output
    console!(log, "Ok2");
    let mut input_div_inner_html = String::new();
    for i in 0..num_inputs {
        let name = &pn.input[i];
        let inner_html = format!(
            r#"<input type="checkbox" id="checkbox_input_i{}">{}  "#,
            i, name
        );
        input_div_inner_html.push_str(&inner_html);
    }

    console!(log, "Ok3");
    let mut output_div_inner_html = String::new();
    for i in 0..num_outputs {
        let name = &pn.output[i];
        let inner_html = format!(
            r#"<input type="checkbox" id="checkbox_output_i{}" disabled="">{}  "#,
            i, name
        );
        output_div_inner_html.push_str(&inner_html);
    }

    console!(log, "Ok4");
    js! {
        var input_div = document.getElementById("top_input");
        input_div.innerHTML = @{input_div_inner_html};
        var output_div = document.getElementById("top_output");
        output_div.innerHTML = @{output_div_inner_html};
    }

    console!(log, "Ok5");
    let get_checkbox_inputs = move || {
        let mut v = vec![];
        for i in 0..num_inputs {
            let i_js = i as i32;
            let checked_raw = js! {
                var che = document.getElementById("checkbox_input_i" + @{i_js});
                if(che == null) return null;
                return che.checked;
            };

            // praise type inference
            let bit = match checked_raw.try_into() {
                Ok(false) => Bit::L,
                Ok(true) => Bit::H,
                Err(_) => Bit::X,
            };

            //console!(log, format!("reading input {}: {:?}", i, checked)); 

            v.push(bit);
        }

        v
    };

    let set_checkbox_outputs = move |outputs: &[Bit]| {
        for i in 0..num_outputs {
            let i_js = i as i32;
            // praise type inference
            let bit = match outputs[i] {
                Bit::L => false,
                Bit::H => true,
                Bit::X => false,
            };

            js! {
                var che = document.getElementById("checkbox_output_i" + @{i_js});
                if(che) {
                    che.checked = @{bit};
                }
            }
        }

    };

    let set_style_output_and_signals = move |signals: &[Vec<Bit>]| {
        // The first element in signals must be c_zero's outputs!
        let mut s = format!("");
        for (c_id, c) in signals.iter().enumerate() {
            for (port_id, x) in c.iter().enumerate() {
                let i = if c_id == 0 {
                    yosys_addr[&ComponentIndex::output(c_id, port_id)]
                } else {
                    yosys_addr[&ComponentIndex::input(c_id, port_id)]
                };
                let color = match *x {
                    Bit::L => "#147014",
                    Bit::H => "#70FF70",
                    Bit::X => "#FF0A0A",
                };
                s.push_str(&format!(".wire_port{}_s0 {{ stroke: {}; stroke-width: 3; }}", i, color));
            }
        }
        js! {
            var stylesheet = document.getElementById("wire_style");
            stylesheet.innerHTML = @{s};
        }
    };

    let counter: TextAreaElement = document().query_selector( "#top_output_debug" ).unwrap().unwrap().try_into().unwrap();

    let main_loop = move |show_debug: bool, show_signals: bool| {
        let input = get_checkbox_inputs();
        let output = c.update(&input);

        set_checkbox_outputs(&output);

        if show_signals {
            let internal = c.internal_inputs().unwrap();
            set_style_output_and_signals(&internal);
        }

        if show_debug {
            let message = format!("{:#?}", c);
            counter.set_value(&message);
        }
    };

    // This triggers the recursion limit
    js! {
        var main_loop = @{main_loop};
        var check_run_forever = document.getElementById("check_run_forever");
        var check_run_step = document.getElementById("check_run_step");
        var check_alive = document.getElementById("check_alive");
        var tick_display = document.getElementById("tick_display");
        var check_show_debug = document.getElementById("check_show_debug");
        var check_show_signals = document.getElementById("check_show_signals");
        var target_ticks_per_second = document.getElementById("target_ticks_per_second");
        var tick = 0;

        function demo() {
            if(check_run_forever.checked || check_run_step.checked) {
                main_loop(check_show_debug.checked, check_show_signals.checked);
                check_run_step.checked = false;
                tick += 1;
                tick_display.value = tick;
            }

            if(check_alive.checked == false) {
                // Stop running
                main_loop.drop(); // Necessary to clean up the closure on Rust's side.
                clearInterval(demo);
            } else {
                /*
                // Can we use setInterval if the function takes more than 1000/30 ms
                // to run? Yes, js is singlethreaded.
                var fps = parseInt(target_ticks_per_second.value, 10);
                if(isNaN(fps)) { fps = 30; }
                setTimeout(demo, 1000/fps);
                */
            }
        }

        setInterval(demo, 1000/30);
    }
}
