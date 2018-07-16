use component::{ComponentIndex, Index, Component, CompIo, Structural, Nand, ConstantBit};
use comphdl1;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CompInfo {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
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

#[derive(Clone, Debug)]
struct Assignments {
    // Each entry in v is a vector of signals that are equivalent:
    // a = b; c = d; will create v = [[a, b], [c, d]]
    // And if we add a = c; it will become v = [[a, b, c, d]]
    v: Vec<Vec<String>>,
}

impl Assignments {
    fn new() -> Self {
        Self { v: vec![], }
    }
    fn add(&mut self, c: &CompInfo) {
        assert!(c.inputs.len() == c.outputs.len(), "unbalanced assignment");
        for (left, right) in c.inputs.iter().zip(c.outputs.iter()) {
            if left == right {
                continue;
            }
            let mut left_pos = None;
            let mut right_pos = None;
            for (i, ass) in self.v.iter().enumerate() {
                for x in ass {
                    if x == left {
                        if left_pos.is_some() {
                            panic!("Duplicate");
                        }
                        left_pos = Some(i);
                    }
                    if x == right {
                        if right_pos.is_some() {
                            panic!("Duplicate");
                        }
                        right_pos = Some(i);
                    }
                }
            }

            match (left_pos, right_pos) {
                (None, None) => {
                    // New group
                    self.v.push(vec![left.to_string(), right.to_string()]);
                }
                (Some(i), None) => {
                    // Push right to group which contains left
                    self.v[i].push(right.to_string());
                }
                (None, Some(i)) => {
                    // Push left to group which contains right 
                    self.v[i].push(left.to_string());
                }
                (Some(i), Some(j)) if i != j => {
                    // Merge groups: a=b with c=d when b=d
                    let (i, j) = if i < j { (i, j) } else { (j, i) };
                    let merge = self.v.swap_remove(j);
                    self.v[i].extend(merge);
                }
                (Some(i), Some(j)) if i == j => {
                    // Do nothing, they are already in the same group
                }
                _ => panic!("I missed something?"),
            }
        }
    }
}

// Global component id
// Unique for each type of component: all the Nands have the same id
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct CompId(usize);

#[derive(Debug, Clone)]
pub struct CompDefinition {
    comp: Vec<CompId>, // global component id, including c_zero
    connections: HashMap<ComponentIndex, Vec<ComponentIndex>>, // connections[local_comp_id][output_id]
    generics: HashMap<usize, (usize, usize)>,
}

impl CompDefinition {
    fn new(components: &HashMap<CompId, CompInfo>,
           comp_id: &HashMap<String, CompId>,
           c_zero: &CompInfo,
           other: &[CompInfo]
    ) -> Result<Self, String> {
        let mut comp = vec![];
        let mut assignments = Assignments::new();
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
            if c_zero.name == c.name {
                return Err(format!("Recursive definition of component {}", name));
            }
            

            // Process assignments
            if c.name == "actually, I'm just an assignment" {
                assignments.add(&c);
                continue;
            }
            println!("Inserting {:#?}", c);
            let c_id = comp_id[&c.name];

            // Verify than number of inputs and outputs match
            // TODO: create is_builtin function
            if ["Nand", "ConstantBit"].contains(&c.name.as_str()) {
                // Builtin gates can have a generic number of inputs or outputs,
                // we dont check them here, but they are checked when creating
                // these components (create_builtin)
            } else {
                if components[&c_id].inputs.len() != c.inputs.len() {
                    return Err(format!("Wrong number of inputs in {} definition: component \
                        {} has {} inputs but {} were supplied",
                        c_zero.name, c.name,
                        components[&c_id].inputs.len(), c.inputs.len())
                    );
                }
                if components[&c_id].outputs.len() != c.outputs.len() {
                    return Err(format!("Wrong number of outputs in {} definition: component \
                        {} has {} outputs but {} were supplied",
                        c_zero.name, c.name,
                        components[&c_id].outputs.len(), c.outputs.len())
                    );
                }
            }

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

        // Apply assignments
        for ass in assignments.v.iter() {
            let ass2 = &ass[0];
            for ass1 in ass.iter().skip(1) {
                println!("Replacing {} with {}", ass1, ass2);
                let x = signals.remove(&ass1).unwrap_or(vec![]);
                signals.entry(&ass2).or_insert(vec![]).extend(x);
            }
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
                return Err(format!("Signal {} is connected to more than one output: {:#?}",
                       s, con));
            }
            // Remove duplicate connections (can be created using assignments)
            let to = to_set.drain().map(|(k, _v)| k).collect();
            if from.len() == 1 {
                connections.insert(from[0].clone(), to);
            } else { // from.len() == 0
                // panic?
            }
        }

        println!("Signals: {:#?}", signals);

        Ok(Self { comp, connections, generics })
    }
}

#[derive(Debug, Clone)]
pub struct ComponentFactory {
    comp_id: HashMap<String, CompId>,
    components: HashMap<CompId, Rc<CompInfo>>,
    comp_def: HashMap<CompId, Rc<CompDefinition>>,
}

impl ComponentFactory {
    fn new(all: Vec<(CompInfo, Vec<CompInfo>)>) -> Result<Self, String> {
        let mut components = HashMap::new();
        let mut comp_id = HashMap::new();
        let mut comp_def = HashMap::new();

        insert_special_components(&mut components, &mut comp_id);
        let mut i = components.len();

        for &(ref c_zero, ref _other) in all.iter() {
            let mut c_zero = c_zero.clone();
            c_zero.verify();
            if let Some(_) = comp_id.get(&c_zero.name) {
                return Err(format!("Redefinition of component {}", c_zero.name));
            }
            comp_id.insert(c_zero.name.clone(), CompId(i));
            components.insert(CompId(i), c_zero);

            i += 1;
        }

        for (c_zero, other) in all {
            let def = CompDefinition::new(&components, &comp_id, &c_zero, &other)?;
            let g_id = comp_id[&c_zero.name];
            comp_def.insert(g_id, def);
        }

        let components = components.into_iter().map(|(k, v)| (k, Rc::new(v))).collect();
        let comp_def = comp_def.into_iter().map(|(k, v)| (k, Rc::new(v))).collect();

        Ok(Self { components, comp_id, comp_def })
    }
    pub fn create_named(&self, name: &str) -> Option<Box<Component>> {
        println!("Creating component {}", name);
        if let Some(c_id) = self.comp_id.get(name) {
            Some(self.create(*c_id))
        } else {
            // This component does not exist
            None
        }
    }
    fn create(&self, c_id: CompId) -> Box<Component> {
        let ref inputs = self.components[&c_id].inputs;
        let ref outputs = self.components[&c_id].outputs;
        let ref name = self.components[&c_id].name;

        println!("Creating component with id {}: {}", c_id.0, name);
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
                println!("DEBUG: Created builtin gate {}", self.components[&new_id].name);
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

        let gate = Structural::new(c, Rc::clone(&self.components[&c_id]));

        Box::new(gate)
    }
    fn create_builtin(&self, c_id: CompId, num_inputs: usize, num_outputs: usize) -> Option<Box<Component>> {
        let ref name = self.components[&c_id].name;

        Some(match name.as_str() {
            "Nand" => {
                assert_eq!(num_outputs, 1);
                Box::new(Nand::new(num_inputs))
            }
            "ConstantBit" => {
                assert_eq!(num_inputs, 0);
                assert_eq!(num_outputs, 3);
                Box::new(ConstantBit::new())
            }
            _ => return None,
        })
    }
}

fn insert_special_components(components: &mut HashMap<CompId, CompInfo>,
                             comp_id: &mut HashMap<String, CompId>) {
    let mut i = components.len();
    components.insert(CompId(i), CompInfo::new("Nand".into(), vec![], vec![])); // TODO
    comp_id.insert("Nand".into(), CompId(i));
    i += 1;
    components.insert(CompId(i), CompInfo::new("ConstantBit".into(), vec![], vec![])); // TODO
    comp_id.insert("ConstantBit".into(), CompId(i));
    //i += 1;
}

pub fn parse_str(bs: &str) -> Result<ComponentFactory, String> {
    let c = comphdl1::FileParser::new().parse(&bs);

    c.map_err(|e| format!("{}", e)).and_then(|c| ComponentFactory::new(c))
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

#[test]
fn wrong_number_of_inputs() {
    let d = r#"
component Or2(a, b) -> x {
    Nand(a) -> n_a;
    Nand(b) -> n_b;
    Nand(n_a, n_b) -> x; 
}

component Or3a(a, b, c) -> x {
    Or2(a, b, c) -> x;
}
component Or3b(a, b, c) -> x {
    Or2(a) -> x;
}
    "#;

    let pd = comphdl1::FileParser::new().parse(d).unwrap();
    let cf = ComponentFactory::new(pd);
    println!("{:#?}", cf);
    assert!(cf.is_err());

    let d = r#"
component Or2(a, b) -> x {
    Nand(a) -> n_a;
    Nand(b) -> n_b;
    Nand(n_a, n_b) -> x; 
}

component Or3(a, b, c) -> x {
    Or2(a, b) -> ab;
    Or2(ab, c) -> x;
}
    "#;
    let pd = comphdl1::FileParser::new().parse(d).unwrap();
    let cf = ComponentFactory::new(pd);
    println!("{:#?}", cf);
    assert!(cf.is_ok());
}

#[test]
fn redefinition() {
    let d = r#"
component Or2(a, b) -> x {
    Nand(a) -> n_a;
    Nand(b) -> n_b;
    Nand(n_a, n_b) -> x; 
}

component Or2(a, b, c) -> x {
    Nand(a, b) -> x;
}
    "#;

    let pd = comphdl1::FileParser::new().parse(d).unwrap();
    let cf = ComponentFactory::new(pd);
    println!("{:#?}", cf);
    assert!(cf.is_err());
}
