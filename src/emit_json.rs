use serde_json;
use component::{Component, Structural, PortNames, Index, ComponentIndex, Direction};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct YosysJson {
    creator: String,
    modules: HashMap<String, Module>,
}

impl YosysJson {
    fn from_structural(c: &Structural) -> Self {
        let mut modules = HashMap::new();
        let m = Module::from_structural(c);
        modules.insert(c.name().to_string(), m);

        Self {
            creator: "comphdl 0.3".to_string(),
            modules,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Module {
    #[serde(serialize_with = "ordered_map")]
    ports: HashMap<String, Port>,
    #[serde(serialize_with = "ordered_map")]
    cells: HashMap<String, Cell>,
    #[serde(serialize_with = "ordered_map")]
    netnames: HashMap<String, Netname>,
}

impl Module {
    fn from_structural(c: &Structural) -> Self {
        let num_components = c.components.len();

        let (ports, pin_addr_to_yosys_addr) = ports_and_addrs(c);

        let mut cells = HashMap::new();

        for c_id in 1..num_components {
            let n_in = c.components[c_id].comp.num_inputs();
            let n_out = c.components[c_id].comp.num_outputs();
            let name = c.components[c_id].comp.name().to_string();
            let connections = &c.components[c_id].connections;
            let port_names = c.components[c_id].comp.port_names();
            let cell = Cell::new(c_id, name, n_in, n_out, connections, &pin_addr_to_yosys_addr, &port_names);
            let name = c.components[c_id].comp.name();
            cells.insert(format!("${}$input.v:1${}", name, c_id), cell);
        }

        Self {
            ports,
            cells,
            netnames: HashMap::new(), // netnames aren't important
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Port {
    direction: Direction,
    bits: Vec<usize>,
}

impl Port {
    fn input(n: usize) -> Port {
        Port {
            direction: Direction::Input,
            bits: vec![n],
        }
    }
    fn output(n: usize) -> Port {
        Port {
            direction: Direction::Output,
            bits: vec![n],
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Cell {
    hide_name: u8,
    #[serde(rename="type")]
    _type: String,
    //parameters
    //attributes
    #[serde(serialize_with = "ordered_map")]
    port_directions: HashMap<String, Direction>,
    #[serde(serialize_with = "vec_into_map")]
    connections: Vec<(String, Vec<usize>)>,
}

impl Cell {
    fn new(c_id: usize, name: String, n_in: usize, n_out: usize,
           connections: &Vec<Vec<Index>>,
           pin_addr_to_yosys_addr: &HashMap<ComponentIndex, usize>,
           port_names: &PortNames
    ) -> Cell {
        let mut port_directions = HashMap::new();
        let mut yosys_connections = vec![];
        for i in 0..n_in {
            let ref pn = port_names.input[i];
            port_directions.insert(format!("{}", pn), Direction::Input);
            let x = ComponentIndex::input(c_id, i);
            let yos_addr = vec![pin_addr_to_yosys_addr[&x]];
            yosys_connections.push((format!("{}", pn), yos_addr));
        }
        for i in 0..n_out {
            let ref pn = port_names.output[i];
            port_directions.insert(format!("{}", pn), Direction::Output);
            //let jj = ComponentIndex::output(c_id, i);
            let mut yos_addr = vec![];
            // TODO: if connections is empty, insert dummy port
            for x in connections[i].iter() {
                let a = if x.comp_id == 0 {
                    ComponentIndex::output(x.comp_id, x.input_id)
                } else {
                    ComponentIndex::input(x.comp_id, x.input_id)
                };
                yos_addr.push(pin_addr_to_yosys_addr[&a]);
            }
            yosys_connections.push((format!("{}", pn), yos_addr));
        }

        Cell {
            hide_name: 0,
            _type: name,
            port_directions,
            connections: yosys_connections,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Netname {
    hide_name: u8,
    bits: Vec<usize>,
    //attributes
}

impl Netname {
    #[allow(dead_code)]
    fn new(n: usize) -> Netname {
        Netname {
            hide_name: 1,
            bits: vec![n],
        }
    }
}

fn ports_and_addrs(c: &Structural) -> (HashMap<String, Port>, HashMap<ComponentIndex, usize>) {
    let mut ports = HashMap::new();
    let num_inputs = c.num_inputs();
    let num_outputs = c.num_outputs();
    let num_components = c.components.len();
    let mut pin_addr_to_yosys_addr: HashMap<ComponentIndex, usize> = HashMap::new();
    let mut ya = 2; // start with address 2 because 0 and 1 are logical 0 and 1
    for i in 0..num_inputs {
        let ref port_name = c.port_names().input[i];
        ports.insert(format!("{}", port_name), Port::input(ya));
        let pa = ComponentIndex::input(0, i);
        pin_addr_to_yosys_addr.insert(pa, ya);
        ya += 1;
    }
    for i in 0..num_outputs {
        let ref port_name = c.port_names().output[i];
        ports.insert(format!("{}", port_name), Port::output(ya));
        let pa = ComponentIndex::output(0, i);
        pin_addr_to_yosys_addr.insert(pa, ya);
        ya += 1;
    }

    for c_id in 1..num_components {
        let n_in = c.components[c_id].comp.num_inputs();
        let n_out = c.components[c_id].comp.num_outputs();

        for j in 0..n_in {
            let pa = ComponentIndex::input(c_id, j);
            pin_addr_to_yosys_addr.insert(pa, ya);
            ya += 1;
        }
        for j in 0..n_out {
            let pa = ComponentIndex::output(c_id, j);
            pin_addr_to_yosys_addr.insert(pa, ya);
            ya += 1;
        }
    }

    // Replace connections from i0 (0) to c2_i0 (12) as c2_i0(0)
    /*
    for i in 0..num_inputs {
        let pa = Index::new(0, i);
        let to = &c.signals[&pa].to;
        let to = c.components[0].connections;
        for x in to {
            *pin_addr_to_yosys_addr.get_mut(&x).unwrap() = pin_addr_to_yosys_addr[&pa];
        }
    }
    */
    for i in 0..num_inputs {
        let pa = ComponentIndex::input(0, i);
        let to = &c.components[0].connections[i];
        for x in to {
            if x.comp_id == 0 {
                let a = ComponentIndex::output(x.comp_id, x.input_id);
                let ref port_name = c.port_names().output[x.input_id];
                let ref port_name_in = c.port_names().input[i];
                let ex = ports[port_name_in].bits.clone();
                ports.get_mut(port_name).unwrap().bits = ex;
                *pin_addr_to_yosys_addr.get_mut(&a).unwrap() = pin_addr_to_yosys_addr[&pa];
            } else {
                let a = ComponentIndex::input(x.comp_id, x.input_id);
                *pin_addr_to_yosys_addr.get_mut(&a).unwrap() = pin_addr_to_yosys_addr[&pa];
            }
        }
    }

    (ports, pin_addr_to_yosys_addr)
}

#[allow(dead_code)]
pub fn yosys_addr_map(c: &Structural) -> HashMap<ComponentIndex, usize> {
    ports_and_addrs(c).1
}

pub fn from_structural(c: &Structural) -> Result<String, serde_json::Error> {
    let cj = YosysJson::from_structural(c);
    let s = serde_json::to_string(&cj)?;

    Ok(s)
}

// https://stackoverflow.com/questions/42723065/how-to-sort-hashmap-keys-when-serializing-with-serde
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;
use std::hash::Hash;
fn ordered_map<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Eq+Hash+Ord+Serialize,
    V: Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

// Preserve insertion order, using a Vec(K, V) and serializing as a Map<K, V>
// https://serde.rs/impl-serialize.html
use serde::ser::SerializeMap;
fn vec_into_map<S, K, V>(value: &Vec<(K, V)>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Serialize,
    V: Serialize,
{
    let mut map = serializer.serialize_map(Some(value.len()))?;
    for &(ref k, ref v) in value {
        map.serialize_entry(k, v)?;
    }
    map.end()
}
