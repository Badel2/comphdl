use serde_json;
use component::{Component, Structural};
use bit::Bit;
use std::cmp;

#[derive(Debug, Serialize, Deserialize)]
pub struct WaveJson {
    signal: Vec<Signal>,
    head: Foot,
    foot: Foot,
    #[serde(skip_serializing)]
    max_buffer_len: usize,
}

impl WaveJson {
    pub fn from_structural(c: &Structural) -> Self {
        let mut signal = vec![];
        // Display inputs and outputs
        let port_names = c.port_names();
        for n in &port_names.input {
            let s = Signal::new(n.clone());
            signal.push(s);
        }
        for n in &port_names.output {
            let s = Signal::new(n.clone());
            signal.push(s);
        }
        let mut head = Foot::new();
        head.set_tick(0);
        let foot = Foot::new();
        let max_buffer_len = usize::max_value();
        Self { signal, head, foot, max_buffer_len }
    }
    pub fn set_buffer_len(&mut self, len: usize) {
        self.max_buffer_len = len;
    }
    // TODO: set_tick, clear
    pub fn update(&mut self, c: &Structural) {
        let inputs = c.input();
        let outputs = c.output();
        let mut n = 0;
        for (i, &x) in inputs.iter().chain(outputs.iter()).enumerate() {
            let s = &mut self.signal[i];
            s.push_value(x);
            n = cmp::max(n, s.keep_last(self.max_buffer_len));
        }

        let tick = self.head.tick.map(|x| x + n as u32);
        self.head.set_tick(tick);
    }
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum SignalOrGroup {
    Value(Signal),
    Group,
    Gap,
}

#[derive(Debug, Serialize, Deserialize)]
struct Signal {
    name: String,
    wave: String,
}

impl Signal {
    fn new(name: String) -> Self {
        Signal { name, wave: "".into() }
    }
    fn push_value(&mut self, value: Bit) {
        let mut c = match value {
            Bit::L => 'l',
            Bit::H => 'h',
            Bit::X => 'X',
        };
        // Get last value in wave different from '.'
        let last_value = self.wave.chars().filter(|&x| x != '.').last().unwrap_or('X');
        // If current value == last value, insert '.'
        if c == last_value {
            c = '.';
        }
        self.wave.push(c);
    }
    fn keep_last(&mut self, n: usize) -> usize {
        let len = self.wave.chars().count();
        if len <= n {
            return 0;
        }
        let wave = {
            let (start, end) = self.wave.split_at(len - n);
            let (endfirst, endother) = end.split_at(1);
            // If the new string begins with a '.', we must fix that and set
            // it to the last symbol
            let mut first = endfirst.chars().next().unwrap();
            if first == '.' {
                // Get last value in wave different from '.'
                let last_value = start.chars().filter(|&x| x != '.').last().unwrap_or('X');
                first = last_value;
            }

            format!("{}{}", first, endother)
        };
        self.wave = wave;

        // Returns the number of removed chars, used to offset the tick scale
        len - n
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Foot {
    text: Option<String>,
    tick: Option<u32>,
    tock: Option<u32>,
}

impl Foot {
    fn new() -> Self {
        Default::default()
    }
    fn set_tick(&mut self, tick: impl Into<Option<u32>>) {
        self.tick = tick.into();
    }
}

#[test]
fn from_or2() {
    use parser;
    const OR2: &str = r#"
    component Or2(a, b) -> x {
        Nand(a) -> n_a;
        Nand(b) -> n_b;
        Nand(n_a, n_b) -> x; 
    }
    "#;
    let cf = parser::parse_str(OR2).unwrap();
    let or2 = cf.create_named("Or2").unwrap();
    let or2 = or2.as_structural().unwrap();
    let mut s = WaveJson::from_structural(or2);
    s.update(or2);
}
