use sim::{nand::Nand, Bit, CompIo, Component, Index, Structural};
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    // test NAND Component
    //let mut nand = Nand::new(2);
    //let output = nand.update(&[Bit::X, Bit::H]);
    //println!("Nand Output: {:?}", output);

    // test OR Component
    //let mut or = Or2::new();
    //let output = or.update(&[Bit::X, Bit::H]);
    //println!("OR Output: {:?}", output);

    let mut buf = Vec::new();
    let mut or = boxed_or_gate();
    let mut input = std::iter::repeat(vec![Bit::L, Bit::L, Bit::L, Bit::H, Bit::H, Bit::L])
        .take(5)
        .chain(std::iter::repeat(vec![Bit::H, Bit::H, Bit::L, Bit::H, Bit::H, Bit::L]).take(5))
        .cycle();

    sim::run_simulation(
        &mut buf,
        &mut or,
        &[vec![Bit::L, Bit::H], vec![Bit::H, Bit::H]],
        2,
    )
    .unwrap();

    let mut vcd_file = File::create("signal.vcd").expect("Cannot create vcd file");
    let _ = vcd_file.write_all(&buf).unwrap();

    Ok(())
}

fn boxed_or_gate() -> Structural {
    let mut c = vec![];
    let mut c_zero = CompIo::c_zero(2, 1); // c_id: 0
    let mut nand_a = CompIo::new(Box::new(Nand::new(1))); // c_id: 1
    let mut nand_b = CompIo::new(Box::new(Nand::new(1))); // c_id: 2
    let mut nand_c = CompIo::new(Box::new(Nand::new(2))); // c_id: 3

    c_zero.add_connection(0, Index::new(1, 0)); // input 0 -> nand_a
    c_zero.add_connection(1, Index::new(2, 0)); // input 1 -> nand_b
    nand_a.add_connection(0, Index::new(3, 0)); // nand_a -> nand_c
    nand_b.add_connection(0, Index::new(3, 1)); // nand_b -> nand_c
    nand_c.add_connection(0, Index::new(0, 0)); // output of nand_c == output of or

    c.push(c_zero);
    c.push(nand_a);
    c.push(nand_b);
    c.push(nand_c);

    Structural::new(c, 2, 1, "OR2")
}
