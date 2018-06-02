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
    let definition = get_element_by_id_value("comphdl_definition");
    let top = get_element_by_id_value("top_name");

    console!(log, "Ok1");
    console!(log, format!("{:?}\n{:?}", definition, top));
    let mut cf = parser::parse_str(&definition);
    let mut c = cf.create_named(&top);
    let cs = c.clone_as_structural().unwrap();
    let s = emit_json::from_structural(&cs).unwrap();

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

    let counter: TextAreaElement = document().query_selector( "#top_output_debug" ).unwrap().unwrap().try_into().unwrap();

    let main_loop = move || {
        let input = get_checkbox_inputs();
        let output = c.update(&input);

        set_checkbox_outputs(&output);

        let message = format!("{:#?}", c);
        counter.set_value(&message);
    };

    js! {
        var main_loop = @{main_loop};

        function demo() {
            main_loop();
            // TODO: how do we stop?
            //main_loop.drop(); // Necessary to clean up the closure on Rust's side.

            setTimeout(demo, 1000/30);
        }

        demo();
    }

}
