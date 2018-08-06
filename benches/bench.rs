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

static MULTIBUF: &str = r#"
component MultiBuf(a) -> (x1, x2, x3, x4, x5, x6) {
    Buf(a) -> x0;
    Buf60(x0) -> x1;
    Buf60(x1) -> x2;
    Buf60(x2) -> x3;
    Buf60(x3) -> x4;
    Buf60(x4) -> x5;
    Buf60(x5) -> x6;
}

component Buf(a) -> x {
    x = a;
}

component Buf4(a) -> x {
    Buf(a) -> a1;
    Buf(a1) -> a2;
    Buf(a2) -> a3;
    Buf(a3) -> x;
}

component Buf10(a) -> x {
    Buf(a) -> a1;
    Buf4(a1) -> a2;
    Buf4(a2) -> a3;
    Buf(a3) -> x;
}

component Buf60(a) -> x {
    Buf10(a) -> a1;
    Buf10(a1) -> a2;
    Buf10(a2) -> a3;
    Buf10(a3) -> a4;
    Buf10(a4) -> a5;
    Buf10(a5) -> x;
}
"#;

static RAM16K: &str = r#"
component Buf (d) -> q { d=q; }
component Not (a) -> q { Nand(a) -> q; }

component And3(a, b, c) -> x {
    Nand(a, b, c) -> n_x;
    Not(n_x) -> x;
}

component Mux_4_1(s1, s0, a, b, c, d) -> y {
    Buf  (s1) -> d_s1;
    Buf  (s0) -> d_s0;
    Not  (s1) -> n_s1;
    Not  (s0) -> n_s0;
    Buf(a) -> d_a; Buf(b) -> d_b; Buf(c) -> d_c; Buf(d) -> d_d;
    Nand (n_s0, n_s1, d_a) -> sel00;
    Nand (d_s0, n_s1, d_b) -> sel01;
    Nand (n_s0, d_s1, d_c) -> sel10;
    Nand (d_s0, d_s1, d_d) -> sel11;
    Nand (sel00, sel01, sel10, sel11) -> y;
}

component Demux_1_4(s1, s0, i) -> (f0, f1, f2, f3) {
    Buf(i)  -> d_i;
    Buf(s1) -> d_s1; Buf(s0) -> d_s0;
    Not(s1) -> n_s1; Not(s0) -> n_s0;
    And3(n_s1, n_s0, d_i) -> f0;
    And3(n_s1, d_s0, d_i) -> f1;
    And3(d_s1, n_s0, d_i) -> f2;
    And3(d_s1, d_s0, d_i) -> f3;
}

component RSLatch_raw(n_R, n_S) -> Q {
    Nand(n_S, n_Q) -> Q;
    Nand(n_R, Q) -> n_Q;
}

component DLatch(enable, d) -> q {
    // set = enable and d
    // reset = enable and not d
    Nand(enable, d) -> n_S;
    Nand(n_S, enable) -> n_R;
    RSLatch_raw(n_R, n_S) -> q;
}

component Register8(enable, d[7:0]) -> q[7:0] {
    DLatch(enable, d[7]) -> q[7];
    DLatch(enable, d[6]) -> q[6];
    DLatch(enable, d[5]) -> q[5];
    DLatch(enable, d[4]) -> q[4];
    DLatch(enable, d[3]) -> q[3];
    DLatch(enable, d[2]) -> q[2];
    DLatch(enable, d[1]) -> q[1];
    DLatch(enable, d[0]) -> q[0];
}

component Register32(enable, d[31:0]) -> q[31:0] {
    Register8(enable, d[31:24]) -> q[31:24];
    Register8(enable, d[23:16]) -> q[23:16];
    Register8(enable, d[15:8]) -> q[15:8];
    Register8(enable, d[7:0]) -> q[7:0];
}

component Mux_32_8(sel[1:0], a[7:0], b[7:0], c[7:0], d[7:0]) -> y[7:0] {
    Mux_4_1(sel[1:0], a[0], b[0], c[0], d[0]) -> y[0];
    Mux_4_1(sel[1:0], a[1], b[1], c[1], d[1]) -> y[1];
    Mux_4_1(sel[1:0], a[2], b[2], c[2], d[2]) -> y[2];
    Mux_4_1(sel[1:0], a[3], b[3], c[3], d[3]) -> y[3];
    Mux_4_1(sel[1:0], a[4], b[4], c[4], d[4]) -> y[4];
    Mux_4_1(sel[1:0], a[5], b[5], c[5], d[5]) -> y[5];
    Mux_4_1(sel[1:0], a[6], b[6], c[6], d[6]) -> y[6];
    Mux_4_1(sel[1:0], a[7], b[7], c[7], d[7]) -> y[7];
}

component Ram4x8(write, addr[1:0], d[7:0]) -> q[7:0] {
    Demux_1_4(addr[1:0], write) -> w[0:3];
    Register8(w[0], d[7:0]) -> qa[7:0];
    Register8(w[1], d[7:0]) -> qb[7:0];
    Register8(w[2], d[7:0]) -> qc[7:0];
    Register8(w[3], d[7:0]) -> qd[7:0];
    Mux_32_8(addr[1:0], qa[7:0], qb[7:0], qc[7:0], qd[7:0]) -> q[7:0];
}

component Ram16x8(write, addr[3:0], d[7:0]) -> q[7:0] {
    Demux_1_4(addr[3:2], write) -> w[0:3];
    Ram4x8(w[0], addr[1:0], d[7:0]) -> qa[7:0];
    Ram4x8(w[1], addr[1:0], d[7:0]) -> qb[7:0];
    Ram4x8(w[2], addr[1:0], d[7:0]) -> qc[7:0];
    Ram4x8(w[3], addr[1:0], d[7:0]) -> qd[7:0];
    Mux_32_8(addr[3:2], qa[7:0], qb[7:0], qc[7:0], qd[7:0]) -> q[7:0];   
}

component Ram64x8(write, addr[5:0], d[7:0]) -> q[7:0] {
    Demux_1_4(addr[5:4], write) -> w[0:3];
    Ram16x8(w[0], addr[3:0], d[7:0]) -> qa[7:0];
    Ram16x8(w[1], addr[3:0], d[7:0]) -> qb[7:0];
    Ram16x8(w[2], addr[3:0], d[7:0]) -> qc[7:0];
    Ram16x8(w[3], addr[3:0], d[7:0]) -> qd[7:0];
    Mux_32_8(addr[5:4], qa[7:0], qb[7:0], qc[7:0], qd[7:0]) -> q[7:0];   
}

component Ram256x8(write, addr[7:0], d[7:0]) -> q[7:0] {
    Demux_1_4(addr[7:6], write) -> w[0:3];
    Ram64x8(w[0], addr[5:0], d[7:0]) -> qa[7:0];
    Ram64x8(w[1], addr[5:0], d[7:0]) -> qb[7:0];
    Ram64x8(w[2], addr[5:0], d[7:0]) -> qc[7:0];
    Ram64x8(w[3], addr[5:0], d[7:0]) -> qd[7:0];
    Mux_32_8(addr[7:6], qa[7:0], qb[7:0], qc[7:0], qd[7:0]) -> q[7:0];   
}

component Ram1024x8(write, addr[9:0], d[7:0]) -> q[7:0] {
    Demux_1_4(addr[9:8], write) -> w[0:3];
    Ram256x8(w[0], addr[7:0], d[7:0]) -> qa[7:0];
    Ram256x8(w[1], addr[7:0], d[7:0]) -> qb[7:0];
    Ram256x8(w[2], addr[7:0], d[7:0]) -> qc[7:0];
    Ram256x8(w[3], addr[7:0], d[7:0]) -> qd[7:0];
    Mux_32_8(addr[9:8], qa[7:0], qb[7:0], qc[7:0], qd[7:0]) -> q[7:0];   
}

component Ram4096x8(write, addr[11:0], d[7:0]) -> q[7:0] {
    Demux_1_4(addr[11:10], write) -> w[0:3];
    Ram1024x8(w[0], addr[9:0], d[7:0]) -> qa[7:0];
    Ram1024x8(w[1], addr[9:0], d[7:0]) -> qb[7:0];
    Ram1024x8(w[2], addr[9:0], d[7:0]) -> qc[7:0];
    Ram1024x8(w[3], addr[9:0], d[7:0]) -> qd[7:0];
    Mux_32_8(addr[11:10], qa[7:0], qb[7:0], qc[7:0], qd[7:0]) -> q[7:0];   
}

component Ram16384x8(write, addr[13:0], d[7:0]) -> q[7:0] {
    Demux_1_4(addr[13:12], write) -> w[0:3];
    Ram4096x8(w[0], addr[11:0], d[7:0]) -> qa[7:0];
    Ram4096x8(w[1], addr[11:0], d[7:0]) -> qb[7:0];
    Ram4096x8(w[2], addr[11:0], d[7:0]) -> qc[7:0];
    Ram4096x8(w[3], addr[11:0], d[7:0]) -> qd[7:0];
    Mux_32_8(addr[13:12], qa[7:0], qb[7:0], qc[7:0], qd[7:0]) -> q[7:0];   
}

component Ram65536x8(write, addr[15:0], d[7:0]) -> q[7:0] {
    Demux_1_4(addr[15:14], write) -> w[0:3];
    Ram16384x8(w[0], addr[13:0], d[7:0]) -> qa[7:0];
    Ram16384x8(w[1], addr[13:0], d[7:0]) -> qb[7:0];
    Ram16384x8(w[2], addr[13:0], d[7:0]) -> qc[7:0];
    Ram16384x8(w[3], addr[13:0], d[7:0]) -> qd[7:0];
    Mux_32_8(addr[15:14], qa[7:0], qb[7:0], qc[7:0], qd[7:0]) -> q[7:0];   
}
"#;

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
