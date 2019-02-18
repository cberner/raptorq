extern crate petgraph;
extern crate primal;
extern crate raptorq;


use raptorq::IntermediateSymbolDecoder;
use raptorq::Octet;
use raptorq::generate_constraint_matrix;
use raptorq::extended_source_block_symbols;
use raptorq::Symbol;

fn main() {
    Octet::static_init();

    for elements in [10, 100, 1000, 10000].iter() {
        let num_symbols = extended_source_block_symbols(*elements);
        let indices: Vec<u32> = (0..num_symbols).collect();
        let a = generate_constraint_matrix(num_symbols, &indices);
        let mut density = 0;
        for i in 0..a.height() {
            for j in 0..a.width() {
                if a.get(i, j) != Octet::zero() {
                    density += 1;
                }
            }
        }
        println!("Original density for {}x{}: {} of {}", a.height(), a.width(), density, a.height() * a.width());

        let symbols = vec![Symbol::zero(1); a.width()];
        let mut decoder = IntermediateSymbolDecoder::new(a, symbols, num_symbols);
        decoder.execute();
        println!("Optimized decoder mul ops: {} ({:.1} per symbol), add ops: {} ({:.1} per symbol)",
                 decoder.get_symbol_mul_ops(),
                 decoder.get_symbol_mul_ops() as f64 / num_symbols as f64,
                 decoder.get_symbol_add_ops(),
                 decoder.get_symbol_add_ops() as f64 / num_symbols as f64);
        println!("By phase mul ops: {:?}, add ops: {:?}",
                 decoder.get_symbol_mul_ops_by_phase(),
                 decoder.get_symbol_add_ops_by_phase());
    }
}