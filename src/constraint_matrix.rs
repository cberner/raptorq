use systematic_constants::extended_source_block_symbols;
use systematic_constants::num_hdpc_symbols;
use systematic_constants::num_ldpc_symbols;
use systematic_constants::num_pi_symbols;
use systematic_constants::num_lt_symbols;
use matrix::OctetMatrix;

// See section 5.3.3.4.2
#[allow(non_snake_case)]
pub fn generate_constraint_matrix(source_block_symbols: u32) -> OctetMatrix {
    let Kprime = extended_source_block_symbols(source_block_symbols);
    let S = num_ldpc_symbols(source_block_symbols);
    let H = num_hdpc_symbols(source_block_symbols);
    let B = num_lt_symbols(source_block_symbols) - S;
    let U = num_pi_symbols(source_block_symbols) - H;

    unimplemented!();
}
