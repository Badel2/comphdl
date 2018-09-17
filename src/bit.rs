use vcd::Value;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Bit {
    L, // Low, false, 0
    H, // High, true, 1
    X, // Undefined
}

impl From<Bit> for Value {
    fn from(x: Bit) -> Self {
        match x {
            Bit::L => Value::V0,
            Bit::H => Value::V1,
            Bit::X => Value::X,
        }
    }
}

impl Bit {
    pub fn from_bool(x: bool) -> Bit {
        if x { Bit::H } else { Bit::L }
    }
    pub fn from_u8(x: u8) -> Vec<Bit> {
        vec![
            Bit::from_bool((x >> 7) & 1 != 0),
            Bit::from_bool((x >> 6) & 1 != 0),
            Bit::from_bool((x >> 5) & 1 != 0),
            Bit::from_bool((x >> 4) & 1 != 0),
            Bit::from_bool((x >> 3) & 1 != 0),
            Bit::from_bool((x >> 2) & 1 != 0),
            Bit::from_bool((x >> 1) & 1 != 0),
            Bit::from_bool((x >> 0) & 1 != 0),
        ]
    }
}

// Returns all the n-bit combinations in order, loops infinitely
#[allow(dead_code)]
pub struct InfiniteInputIterator {
    current: Vec<Bit>,
}

impl InfiniteInputIterator {
    #[allow(dead_code)]
    pub fn new(n: usize) -> Self {
        Self { current: vec![Bit::L; n] }
    }
}

impl Iterator for InfiniteInputIterator {
    type Item = Vec<Bit>;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.current.clone();
        self.current = next_bit_combination(&a);

        Some(a)
    }
}

#[derive(Debug, Clone)]
pub struct RepInputIterator {
    current: Vec<Bit>,
    count: u32,
    max_count: u32,
}

impl RepInputIterator {
    pub fn new(n: usize, rep: u32) -> Self {
        Self {
            current: vec![Bit::L; n],
            count: rep,
            max_count: rep,
        }
    }
}

impl Iterator for RepInputIterator {
    type Item = Vec<Bit>;

    fn next(&mut self) -> Option<Vec<Bit>> {
        let a = self.current.clone();
        self.count -= 1;
        if self.count == 0 {
            self.current = next_bit_combination(&a);
            self.count = self.max_count;
        }

        Some(a)
    }
}

fn next_bit_combination(x: &[Bit]) -> Vec<Bit> {
    let mut y = x.to_vec();
    let mut i = x.len();
    let mut carry = true;
    while carry == true && i > 0 {
        i -= 1;
        match x[i] {
            Bit::L => { carry = false; y[i] = Bit::H; },
            Bit::H => { carry = true; y[i] = Bit::L; },
            // Incrementing X would make all the higher bits X, we don't want
            // that, instead we keep it as X and increment the next bit
            Bit::X => { carry = true; },
        }
    }

    y
}

#[test]
fn inf_equals_rep_1() {
    let mut inf = InfiniteInputIterator::new(4);
    let mut rep = RepInputIterator::new(4, 1);

    for _ in 0..20 {
        assert_eq!(inf.next(), rep.next());
    }
}

#[test]
fn inf_equals_rep_2() {
    let mut inf = InfiniteInputIterator::new(4);
    let mut rep = RepInputIterator::new(4, 2);

    for _ in 0..20 {
        rep.next();
        assert_eq!(inf.next(), rep.next());
    }
}
