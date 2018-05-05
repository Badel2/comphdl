use std::io::{BufReader, Read};
use std::fs::File;
use std::collections::HashMap;
use super::{comphdl1, Index, Component, CompIo, PortNames, Structural, Nand, ConstantBit};
use emit_json::ComponentIndex;

#[derive(Debug, Clone)]
pub struct CompInfo {
    name: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl CompInfo {
    pub fn new(name: String, inputs: Vec<String>, outputs: Vec<String>) -> Self {
        CompInfo {
            name, inputs, outputs,
        }
    }
    pub fn verify(&mut self) {
        let mut repetitions = HashMap::new();
        for s in self.inputs.iter() {
            if let Some(_) = repetitions.get(s) {
                panic!("Input names must be unique, {}.{} isn't", self.name, s);
            }
            repetitions.insert(s, ());
        }

        // Output names must also be unique, example: 1-to-4
        // quad(a) -> (a, a, a, a)
        // The alternative is
        // quad(a) -> (a0, a1, a2, a3) { a0 = a; a1 = a; a2 = a; a3 = a; }
        for s in self.outputs.iter() {
            if let Some(_) = repetitions.get(s) {
                panic!("Output names must be unique, '{}.{}' isn't", self.name, s);
            }
            repetitions.insert(s, ());
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompDefinition {
    comp: Vec<usize>, // global component id, including c_zero
    connections: HashMap<ComponentIndex, Vec<ComponentIndex>>, // connections[local_comp_id][output_id]
    generics: HashMap<usize, (usize, usize)>,
}

impl CompDefinition {
    fn new(components: &[CompInfo],
           comp_id: &HashMap<String, usize>,
           c_zero: &CompInfo,
           other: &[CompInfo]
    ) -> Self {
        let mut comp = vec![];
        let mut assignments: HashMap<String, String> = HashMap::new();
        let mut signals = HashMap::new();
        let ref name = c_zero.name;

        for c in [c_zero].iter() {
            println!("Inserting {:#?}", c);
            let c_id = comp_id[&c.name];
            comp.push(c_id);
            let l_id = comp.len() - 1;
            // Remember the weirdness: inputs = outputs
            for (j, n) in c.inputs.iter().enumerate() {
                let idx = ComponentIndex::output(l_id, j);
                signals.entry(n).or_insert(vec![]).push(idx);
            }
            for (j, n) in c.outputs.iter().enumerate() {
                let idx = ComponentIndex::input(l_id, j);
                signals.entry(n).or_insert(vec![]).push(idx);
            }
        }

        // If a gate can have an undefined number of inputs, store it here
        let mut generics = HashMap::new();

        for c in other {
            // Prevent recursive definitions
            assert!(name != &c.name, format!("Recursive definition of component {}", name));
            if c.name == "actually, I'm just an assignment" {
                assert!(c.inputs.len() == c.outputs.len(), "unbalanced assignment");
                for (left, right) in c.inputs.iter().zip(c.outputs.iter()) {
                    let replace_this;
                    let replace_with; 
                    // from: to
                    // a: b
                    // b: c => c: b
                    if let Some(a) = assignments.get(left) {
                        // Input has a replacement, replace output with that
                        //assignments.insert(right, a.to_string());
                        replace_this = right.to_string();
                        replace_with = a.to_string();
                    } else if let Some(b) = assignments.get(right) {
                        // Output has a replacement, replace input with that
                        //assignments.insert(left, b.to_string());
                        replace_this = left.to_string();
                        replace_with = b.to_string();
                    } else {
                        //assignments.insert(left, right);
                        replace_this = left.to_string();
                        replace_with = right.to_string();
                    }

                    assignments.insert(replace_this, replace_with);
                }
                continue;
            }
            println!("Inserting {:#?}", c);
            let c_id = comp_id[&c.name];
            comp.push(c_id);
            let l_id = comp.len() - 1;
            generics.insert(l_id, (c.inputs.len(), c.outputs.len()));
            for (j, n) in c.inputs.iter().enumerate() {
                let idx = ComponentIndex::input(l_id, j);
                signals.entry(n).or_insert(vec![]).push(idx);
            }
            for (j, n) in c.outputs.iter().enumerate() {
                let idx = ComponentIndex::output(l_id, j);
                signals.entry(n).or_insert(vec![]).push(idx);
            }
        }

        /*
        // Apply assignments
        for ass in assignments {
            /*
            // Doesn't work u_u
            if let Some(x) = signals.remove(ass1) {
                signals.entry(ass2).or_insert(vec![]).extend(x);
            } else if let Some(x) = signals.remove(ass2) {
                signals.entry(ass1).or_insert(vec![]).extend(x);
            }
            */

        }
        */
        for (ass1, ass2) in assignments.iter() {
            println!("Replacing {} with {}", ass1, ass2);
            let x = signals.remove(&ass1).unwrap_or(vec![]);
            signals.entry(&ass2).or_insert(vec![]).extend(x);
        }
        

        let mut connections = HashMap::with_capacity(signals.len());

        // Verify that each signal is connected to at most one output
        for (s, con) in &signals {
            let mut from = vec![];
            let mut to_set = HashMap::with_capacity(con.len());
            for x in con {
                if x.is_output() {
                    from.push(x.clone());
                } else {
                    //to.push(x.clone());
                    to_set.insert(x.clone(), ());
                }
            }
            if from.len() > 1 {
                panic!("Signal {} is connected to more than one output: {:#?}",
                       s, con);
            }
            // Remove duplicate connections (can be created using assignments)
            let to = to_set.drain().map(|(k, v)| k).collect();
            if from.len() == 1 {
                connections.insert(from[0].clone(), to);
            } else { // from.len() == 0
                // panic?
            }
        }

        println!("Signals: {:#?}", signals);

        Self { comp, connections, generics }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentFactory {
    components: Vec<CompInfo>,
    comp_id: HashMap<String, usize>,
    comp_def: HashMap<usize, CompDefinition>,
}

impl ComponentFactory {
    fn new(all: Vec<(CompInfo, Vec<CompInfo>)>) -> Self {
        let mut components = vec![];
        let mut comp_id = HashMap::new();
        let mut comp_def = HashMap::new();

        insert_special_components(&mut components, &mut comp_id);
        let mut i = components.len();

        for &(ref c_zero, ref other) in all.iter() {
            let mut c_zero = c_zero.clone();
            c_zero.verify();
            if let Some(_) = comp_id.get(&c_zero.name) {
                panic!("Component name already exists");
            }
            comp_id.insert(c_zero.name.clone(), i);
            components.push(c_zero);

            i += 1;
        }

        for (c_zero, other) in all {
            let def = CompDefinition::new(&components, &comp_id, &c_zero, &other);
            let g_id = comp_id[&c_zero.name];
            comp_def.insert(g_id, def);
        }

        Self { components, comp_id, comp_def }
    }
    fn create_named(&self, name: &str) -> Box<Component> {
        println!("Creating component {}", name);
        let c_id = self.comp_id.get(name).expect("This component does not exist");
        self.create(*c_id)
    }
    fn create(&self, c_id: usize) -> Box<Component> {
        let ref inputs = self.components[c_id].inputs;
        let ref outputs = self.components[c_id].outputs;
        let ref name = self.components[c_id].name;

        println!("Creating component with id {}: {}", c_id, name);
        let ref def = self.comp_def[&c_id];

        let c_zero = CompIo::c_zero(inputs.len(), outputs.len());
        let mut c = vec![c_zero];
        
        for (local_id, &new_id) in def.comp.iter().enumerate().skip(1) {
            // We must check that the local definition and the global one
            // have the same number of inputs and outputs

            // Prevent recursive definitions
            //assert!(&self.components[new_id].name != name);
            let (num_i, num_o) = def.generics[&local_id];
            let boxed_gate = if let Some(c) = self.create_builtin(new_id, num_i, num_o) {
                println!("DEBUG: Created builting gate {}", self.components[new_id].name);
                c
            } else {
                self.create(new_id)
            };
            let mut x = CompIo::new(boxed_gate);
            c.push(x);
        }

        for (from, to) in &def.connections {
            let ref mut x = c[from.c_id];
            assert!(from.is_output());
            for ref to in to {
                assert!(!to.is_output());
                x.add_connection(from.port_id, Index::new(to.c_id, to.port_id));
            }
        }

        let pn = PortNames::new_vec(inputs.clone(), outputs.clone());
        let gate = Structural::new(c, inputs.len(), outputs.len(), &self.components[c_id].name, pn);

        Box::new(gate)
    }
    fn create_builtin(&self, c_id: usize, num_inputs: usize, num_outputs: usize) -> Option<Box<Component>> {
        let ref name = self.components[c_id].name;

        Some(match name.as_str() {
            "Nand" => Box::new(Nand::new(num_inputs)),
            "ConstantBit" => Box::new(ConstantBit::new()),
            _ => return None,
        })
    }
}

fn insert_special_components(components: &mut Vec<CompInfo>,
                             comp_id: &mut HashMap<String, usize>) {
    let mut i = components.len();
    components.push(CompInfo::new("Nand".into(), vec![], vec![])); // TODO
    comp_id.insert("Nand".into(), i);
    i += 1;
    components.push(CompInfo::new("ConstantBit".into(), vec![], vec![])); // TODO
    comp_id.insert("ConstantBit".into(), i);
    i += 1;
}

pub fn parse_file(filename: &str, top: &str) -> Box<Component> {
    let file = File::open(filename).expect("Unable to open file");
    let mut reader = BufReader::new(file);
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer).unwrap();
    let bs = String::from_utf8(buffer).unwrap();
    /*
    let parsed = comphdl1::FileParser::new().parse(&bs).unwrap();
    for c in parsed {
        println!("{:?}", c.0);
        for sub_c in c.1 {
            println!("> {:?}", sub_c);
        }
    }
    panic!("Thank you for playing!");
    */
    let c = comphdl1::FileParser::new().parse(&bs).unwrap();
    let s = ComponentFactory::new(c);
    let mux = s.create_named(top);
    println!("{:#?}", mux);

    mux
}

#[test]
fn compdef_test() {
    let d = vec![
        "component Or2(a, b) -> x {}",
        "component Or2(a, b) -> (x) { }",
        "component Or2(a, b) -> (x,) { }",
        "component Or3(a, b, c) -> x { Or2(a, b) -> y; Or2(y, c) -> x; }",
    ];
    
    for x in d {
        println!("{:#?}", comphdl1::CompDefParser::new().parse(x).unwrap());
    }
}
