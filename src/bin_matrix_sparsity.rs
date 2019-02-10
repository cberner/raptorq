extern crate petgraph;
extern crate primal;
extern crate raptorq;

mod arraymap;
mod octets;
#[allow(dead_code)]
mod systematic_constants;
#[allow(dead_code)]
mod rng;
#[allow(dead_code)]
mod octet;
#[allow(dead_code)]
mod symbol;
#[allow(dead_code)]
mod matrix;
#[allow(dead_code)]
mod constraint_matrix;
#[allow(dead_code)]
mod base;

use constraint_matrix::generate_constraint_matrix;
use octet::Octet;
use systematic_constants::extended_source_block_symbols;
use symbol::Symbol;
use base::IntermediateSymbolDecoder;

fn main() {
    Octet::static_init();

    for elements in [10, 100, 1000, 10000].iter() {
        let num_symbols = extended_source_block_symbols(*elements);
        let a = generate_constraint_matrix(num_symbols, 0..num_symbols);
        let mut density = 0;
        for i in 0..a.height() {
            for j in 0..a.width() {
                if a.get(i, j) != Octet::zero() {
                    density += 1;
                }
            }
        }
        println!("Original density for {}x{}: {} of {}", a.height(), a.width(), density, a.height() * a.width());

        let inverse = a.inverse().unwrap();
        let mut density = 0;
        for i in 0..inverse.height() {
            for j in 0..inverse.width() {
                if inverse.get(i, j) != Octet::zero() {
                    density += 1;
                }
            }
        }
        println!("Inverse density for {}x{}: {} of {}", inverse.height(), inverse.width(), density, inverse.height() * inverse.width());

        let symbols = vec![Symbol::zero(1); a.width()];
        let mut decoder = IntermediateSymbolDecoder::new(&a, &symbols, num_symbols);
        decoder.execute();
        println!("Optimized decoder mul ops: {} ({:.1} per symbol), add ops: {} ({:.1} per symbol)",
                 decoder.get_symbol_mul_ops(),
                 decoder.get_symbol_mul_ops() as f64 / num_symbols as f64,
                 decoder.get_symbol_add_ops(),
                 decoder.get_symbol_add_ops() as f64 / num_symbols as f64);
    }
}