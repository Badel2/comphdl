#![feature(test)]

extern crate test;
extern crate comphdl;

use self::test::Bencher;
use comphdl::parser;
use comphdl::bit::{Bit, RepInputIterator};

static OR2: &str = r#"
component Or2(a, b) -> x {
    Nand(a) -> n_a;
    Nand(b) -> n_b;
    Nand(n_a, n_b) -> x; 
}
"#;

static MULTIBUF: &str = include_str!("../../static/comphdl_examples/bufbufbuf.txt"); 

static RAM16K: &str = include_str!("../../static/comphdl_examples/ram.txt");

static CAT: &str = include_str!("../../static/comphdl_examples/cat.txt");

#[bench]
fn parse_null(b: &mut Bencher) {
    b.iter(|| {
        let cf = parser::parse_str("").unwrap();
        cf
    });
}

#[bench]
fn parse_or2(b: &mut Bencher) {
    b.iter(|| {
        let cf = parser::parse_str(OR2).unwrap();
        cf
    });
}

#[bench]
fn parse_multibuf(b: &mut Bencher) {
    b.iter(|| {
        let cf = parser::parse_str(MULTIBUF).unwrap();
        cf
    });
}

#[bench]
fn parse_ram16k(b: &mut Bencher) {
    b.iter(|| {
        let cf = parser::parse_str(RAM16K).unwrap();
        cf
    });
}

#[bench]
fn parse_cat(b: &mut Bencher) {
    b.iter(|| {
        let cf = parser::parse_str(CAT).unwrap();
        cf
    });
}

#[bench]
fn simulate_null(b: &mut Bencher) {
    let cf = parser::parse_str(OR2).unwrap();
    let or2 = cf.create_named("Or2").unwrap();
    b.iter(|| {
        let c = or2.clone();
        c
    });
}

#[bench]
fn simulate_or2(b: &mut Bencher) {
    let cf = parser::parse_str(OR2).unwrap();
    let or2 = cf.create_named("Or2").unwrap();
    let ticks = 1000;
    b.iter(|| {
        let mut c = or2.clone();
        let inputs = RepInputIterator::new(2, 10);
        for current_input in inputs.take(ticks) {
            let _outputs = c.update(&current_input);
        }
        c
    });
}

#[bench]
fn simulate_multibuf(b: &mut Bencher) {
    let cf = parser::parse_str(MULTIBUF).unwrap();
    let c = cf.create_named("MultiBuf").unwrap();
    let ticks = 1000;
    b.iter(|| {
        let mut c = c.clone();
        let inputs = RepInputIterator::new(1, 10);
        for current_input in inputs.take(ticks) {
            let _outputs = c.update(&current_input);
        }
        c
    });
}

#[bench]
fn simulate_multibuf_static_input(b: &mut Bencher) {
    let cf = parser::parse_str(MULTIBUF).unwrap();
    let mut c = cf.create_named("MultiBuf").unwrap();
    let ticks = 1000;
    let input = vec![Bit::L];
    // Warm up
    for _ in 0..ticks {
        let _outputs = c.update(&input);
    }
    // This should be practically a no-op
    b.iter(|| {
        let mut outputs = vec![];
        for _ in 0..ticks {
            outputs = c.update(&input);
        }
        outputs
    });
}

#[bench]
fn simulate_ram16k(b: &mut Bencher) {
    let cf = parser::parse_str(RAM16K).unwrap();
    let mut c = cf.create_named("Ram16384x8").unwrap();
    let ticks = 50;
    let mut inputs = RepInputIterator::new(14, ticks);
    // Write at one address per iter
    b.iter(|| {
        let mut outputs = vec![];
        for _ in 0..ticks {
            let mut inputs_write = vec![Bit::H];
            let current_addr = inputs.next().unwrap();
            inputs_write.extend(&current_addr);
            inputs_write.extend(&[Bit::L; 8]);
            outputs = c.update(&inputs_write);
        }
        outputs
    });
}

#[bench]
fn simulate_cat(b: &mut Bencher) {
    let input = format!("Hello, world! Hmmmmmm... 0123456789 ");
    let input = format!("{}{}{}{}", input, input, input, input);
    let input = format!("{}{}{}{}", input, input, input, input);
    let input = input.into_bytes();
    assert_eq!(input.len(), 576);
    let cf = parser::parse_str(CAT);
    println!("{:#?}", cf);
    let mut cf = cf.unwrap();
    b.iter(|| {
        let mut ticks = 0;
        let out = vec![];

        // We must recreate the component at each iteration because
        // we need to reset the stdin and stdout buffers
        cf.set_stdin_vec(input.clone());
        let handle = cf.set_stdout_vec(out);
        let s = cf.create_named("Cat");
        //println!("{:#?}", s);
        let s = s.unwrap();
        assert_eq!(s.num_inputs(), 1);
        assert_eq!(s.num_outputs(), 1);
        let mut s = s;
        // We read on rising edge, so first set to 0
        for _ in 0..1 {
            assert_eq!(s.update(&[Bit::L]), vec![Bit::X]);
            ticks += 1;
        }
        let mut eof = Bit::L;
        while eof != Bit::H { // antipattern
            eof = s.update(&[Bit::H])[0];
            ticks += 1;
        }
        assert_eq!(&input, handle.borrow_mut().get_ref());
        assert_eq!(ticks, input.len() * 2 + 1 + 3 + 3);
        s
    });
}
