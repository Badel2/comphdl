use comphdl::{emit_json, parser};
use comphdl::component::{ComponentIndex, Structural};
use comphdl::bit::Bit;
use comphdl::wave_json::{WaveJson};
use stdweb::js_export;
use stdweb::web::html_element::TextAreaElement;
use stdweb::web::document;
use stdweb::web::IParentNode;
use stdweb::unstable::TryInto;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, BufRead, Cursor, Read, Write};
use std::cmp;

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

struct ValueReader {
    id: String,
    buf: Cursor<Vec<u8>>,
}

impl ValueReader {
    fn new(id: String) -> Self {
        Self { id, buf: Cursor::new(Vec::new()) }
    }
}

impl Read for ValueReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.fill_buf()?;
        self.buf.read(buf)
    }
}

impl BufRead for ValueReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let s = get_element_by_id_value(&self.id);
        let old_pos = self.buf.position();
        self.buf = Cursor::new(s.as_bytes().to_vec());
        if old_pos <= self.buf.get_ref().len() as u64 {
            // panic?
        }
        self.buf.set_position(old_pos);
        let amt = cmp::min(old_pos, self.buf.get_ref().len() as u64);
        Ok(&self.buf.get_ref()[(amt as usize)..])
    }
    fn consume(&mut self, n: usize) {
        let old_pos = self.buf.position();
        self.buf.set_position(old_pos + n as u64);
    }
}

struct ValueWriter {
    id: String,
    buf: Vec<u8>,
}

impl ValueWriter {
    fn new(id: String) -> Self {
        js! {
            var t = document.getElementById(@{&id});
            if(t != null) {
                t.value = "";
            }
            import("/index.js").then(function(p) {
                p.term.clear();
                p.term2.clear();
            });
        }
        Self { id, buf: vec![] }
    }
}

impl Write for ValueWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let id = &self.id;
        // We always return the buf len, meaning that the entire buffer was
        // written. This may not be true, as this writter has a small internal
        // buffer for dealing with incomplete UTF8 characters.
        let buf_len = buf.len();
        // If the last few bytes of the buffer cannot be converted to a string,
        // it may be because it's an incomplete unicode character.
        // In that case, we save that bytes in a buffer and wait for further
        // input.
        let buf = if self.buf.is_empty() {
            buf.to_vec()
        } else {
            let mut b = self.buf.clone();
            b.extend(buf);
            self.buf.clear();
            b
        };
        // Now buf is [self.buf + buf]
        let utf8_s = String::from_utf8(buf);
        let bufstring = match utf8_s {
            Ok(s) => s,
            Err(err) => {
                let e = err.utf8_error();
                let buf = err.into_bytes();
                match e.error_len() {
                    // The string is valid until the end, so the last character
                    // is incomplete and will be written in the next call
                    None => {
                        let i = e.valid_up_to();
                        // Save last bytes
                        self.buf = buf[i..].to_vec();
                        String::from_utf8(buf[..i].to_vec()).unwrap()
                    }
                    // An invalid character in the middle of the stream, we
                    // just fallback to from_utf8_lossy
                    Some(_) => {
                        String::from_utf8_lossy(&buf).into()
                    }
                }
            }
        };
        //let bufstring: String = String::from_utf8_lossy(buf).into();
        js! {
            var t = document.getElementById(@{id});
            if(t != null) {
                t.value += @{&bufstring};
                // Scroll to bottom
                t.scrollTop = t.scrollHeight;
            }
        }
        // Fix newlines...
        let bufstring = bufstring.replace("\n", "\r\n");
        js! {
            import("/index.js").then(function(p) {
                p.term.write(@{bufstring});
            });
        }

        // We can't fail
        Ok(buf_len)
    }
    fn flush(&mut self) -> io::Result<()> {
        // TODO: should we write self.buf here?
        // Note that self.buf is invalid unicode, so it will be printed as
        // garbage.
        Ok(())
    }
}

#[js_export]
pub fn run_js_gui(definition: &str) -> String {
    debug!("Hello from Rust!");
    // TODO: check if already running
    let top = get_element_by_id_value("top_name");

    let mut cf = match parser::parse_str(&definition) {
        Ok(cf) => cf,
        Err(e) => {
            return format!("Error parsing source code: {}", e);
        }
    };

    debug!("Creating component {}...", top);
    //let stdin_bufread = get_element_by_id_value("stdin_bufread");
    //cf.set_stdin_vec(stdin_bufread.into_bytes());
    cf.set_stdin_bufread(Rc::new(RefCell::new(ValueReader::new("stdin_bufread".into()))));
    cf.set_stdout_bufwrite(Rc::new(RefCell::new(ValueWriter::new("stdout_bufwrite".into()))));

    let mut c = match cf.create_named(&top) {
        Some(c) => c,
        None => {
            if top == "" {
                return format!("You must specify a top component name");
            }
            return format!("Top component `{}` not found", top);
            // TODO: did you mean ...? (find components with similar names)
        }
    };

    debug!("Generating netlist and wavejson...");
    // Borrow the component as a structural to generate the netlist
    let (s, yosys_addr, mut wave_json) = {
        let cs = c.as_structural().unwrap();
        let s = emit_json::from_structural(&cs).unwrap();
        let yosys_addr = emit_json::yosys_addr_map(&cs);
        let mut wave_json = WaveJson::from_structural(cs);
        wave_json.set_buffer_len(50);

        (s, yosys_addr, wave_json)
    };

    debug!("Writing netlist to textarea...");
    let comphdl_json: TextAreaElement = document().query_selector( "#comphdl_json" ).unwrap().unwrap().try_into().unwrap();
    comphdl_json.set_value(&s);

    let num_inputs = c.num_inputs();
    let num_outputs = c.num_outputs();
    let pn = c.port_names();
    
    // Create checkboxes for input and output
    debug!("Creating checkboxes for input and output...");
    let mut input_div_inner_html = String::new();
    for i in 0..num_inputs {
        let name = &pn.input[i];
        let inner_html = format!(
            r#"<input type="checkbox" id="checkbox_input_{}">{}  "#,
            name, name
        );
        input_div_inner_html.push_str(&inner_html);
    }

    let mut output_div_inner_html = String::new();
    for i in 0..num_outputs {
        let name = &pn.output[i];
        let inner_html = format!(
            r#"<input type="checkbox" id="checkbox_output_i{}" disabled="">{}  "#,
            i, name
        );
        output_div_inner_html.push_str(&inner_html);
    }

    js! {
        var input_div = document.getElementById("top_input");
        input_div.innerHTML = @{input_div_inner_html};
        var output_div = document.getElementById("top_output");
        output_div.innerHTML = @{output_div_inner_html};
    }


    let inputnames = pn.input.clone();
    let outputnames = pn.output.clone();
    let get_checkbox_inputs = move || {
        let mut v = vec![];
        for i in 0..num_inputs {
            let name = &inputnames[i];
            let checked_raw = js! {
                var che = document.getElementById("checkbox_input_" + @{name});
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

    let set_checkbox_outputs = move |outputs: &[Bit], old_outputs: &Option<Vec<Bit>>| {
        for i in 0..num_outputs {
            // Skip update if output has not changed
            if let Some(old) = old_outputs {
                if old[i] == outputs[i] {
                    continue;
                }
            }
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

            // Color svg output ports
            let bitchar = match outputs[i] {
                Bit::L => 'L',
                Bit::H => 'H',
                Bit::X => 'X',
            };
            let color = format!("var(--comphdl-wire-color-{})", bitchar);
            let name = &outputnames[i];

            js! {
                var a = document.getElementById("outputExt_" + @{name});
                if(a) {
                    a.style = "fill: " + @{color};
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
                let bitchar = match *x {
                    Bit::L => 'L',
                    Bit::H => 'H',
                    Bit::X => 'X',
                };
                let wire_port_id = format!(".wire_port{}_s0", i);
                let color = format!("var(--comphdl-wire-color-{})", bitchar);
                let width = format!("var(--comphdl-wire-width-{})", bitchar);

                // TODO: performance implications of js macro inside for loop
                js! {
                    var x = document.querySelectorAll(@{wire_port_id});
                    for (var i = 0; i < x.length; i++) {
                        x[i].style.stroke = @{color};
                        x[i].style.strokeWidth = @{width};
                    }
                }
            }
        }
    };

    let mut set_wave_json = move |c: &Structural| {
        // WaveJSON for WaveDrom
        wave_json.update(c);
        let s = wave_json.to_json().unwrap();
        js! {
            var txt = document.getElementById("InputJSON_0");
            txt.value = @{s};
        }
    };

    let counter: TextAreaElement = document().query_selector( "#top_output_debug" ).unwrap().unwrap().try_into().unwrap();

    let mut old_output = None;
    let mut old_internal = None;
    let main_loop = move |show_debug: bool, show_signals: bool, monitor_signals: bool| {
        let input = get_checkbox_inputs();
        let output = c.update(&input);

        set_checkbox_outputs(&output, &old_output);

        if show_signals {
            let internal = c.internal_inputs().unwrap();
            // Skip update if the internal signals have not changed
            //if old_internal.is_none() || old_internal.as_ref().unwrap() != &internal {
            if old_internal.as_ref().map_or(true, |o| o != &internal) {
                set_style_output_and_signals(&internal);
            }
            if monitor_signals {
                set_wave_json(c.as_structural().unwrap());
            } else {
                // Erasing the wave json is not supported yet
                // Remember to update tick
            }
            old_internal = Some(internal);
        }

        if show_debug {
            let message = format!("{:#?}", c);
            counter.set_value(&message);
        }

        old_output = Some(output);
    };

    info!("Rust code run successfully. Setting main loop from js.");
    js! {
        var main_loop = @{main_loop};
        import("/index.js").then(function(p) {
            p.register_main_loop(main_loop);
        });
    }

    return "Everything ok".into();
}
