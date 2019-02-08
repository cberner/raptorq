use systematic_constants::extended_source_block_symbols;
use systematic_constants::num_hdpc_symbols;
use systematic_constants::num_ldpc_symbols;
use systematic_constants::num_pi_symbols;
use systematic_constants::num_lt_symbols;
use systematic_constants::num_intermediate_symbols;
use matrix::OctetMatrix;
use octet::Octet;
use rng::rand;
use std::collections::HashSet;
use systematic_constants::calculate_p1;
use base::intermediate_tuple;

// Generates the GAMMA matrix
// See section 5.3.3.3
#[allow(non_snake_case)]
fn generate_gamma(Kprime: usize, S: usize) -> OctetMatrix {
    let size = Kprime + S;
    let mut matrix = OctetMatrix::new(size, size);
    for i in 0..size {
        for j in 0..=i {
            matrix.set(i, j, Octet::alpha((i - j) as u8));
        }
    }
    matrix
}

// Generates the MT matrix
// See section 5.3.3.3
#[allow(non_snake_case)]
fn generate_mt(H: usize, Kprime: usize, S: usize) -> OctetMatrix {
    let mut matrix = OctetMatrix::new(H, Kprime + S);
    for i in 0..H {
        for j in 0..=(Kprime + S - 2) {
            if i == rand((j + 1) as u32, 6, H as u32) as usize ||
                i == ((rand((j + 1) as u32, 6, H as u32) + rand((j + 1) as u32, 7, (H - 1) as u32) + 1) % (H as u32)) as usize {
                matrix.set(i, j, 1.into());
            }
        }
        matrix.set(i, Kprime + S - 1, Octet::alpha(i as u8));
    }
    matrix
}

// Simulates Enc[] function to get indices of accessed intermediate symbols, as defined in section 5.3.5.3
pub fn enc_indices(source_block_symbols: u32,
       source_tuple: (u32, u32, u32, u32, u32, u32)) -> HashSet<usize> {
    let w = num_lt_symbols(source_block_symbols);
    let p = num_pi_symbols(source_block_symbols);
    let p1 = calculate_p1(source_block_symbols);
    let (d, a, mut b, d1, a1, mut b1) = source_tuple;

    assert!(1 <= a && a < w);
    assert!(b < w);
    assert!(d1 == 2 || d1 == 3);
    assert!(1 <= a1 && a < w);
    assert!(b1 < w);

    let mut indices = HashSet::new();
    indices.insert(b as usize);

    for _ in 1..d {
        b = (b + a) % w;
        indices.insert(b as usize);
    }

    while b1 >= p {
        b1 = (b1 + a1) % p1;
    }

    indices.insert((w + b1) as usize);

    for _ in 1..d1 {
        b1 = (b1 + a1) % p1;
        while b1 >= p {
            b1 = (b1 + a1) % p1;
        }
        indices.insert((w + b1) as usize);
    }

    indices
}

// See section 5.3.3.4.2
#[allow(non_snake_case)]
pub fn generate_constraint_matrix<T:Iterator<Item=u32>>(source_block_symbols: u32, encoded_symbol_indices: T) -> OctetMatrix {
    let Kprime = extended_source_block_symbols(source_block_symbols) as usize;
    let S = num_ldpc_symbols(source_block_symbols) as usize;
    let H = num_hdpc_symbols(source_block_symbols) as usize;
    let W = num_lt_symbols(source_block_symbols) as usize;
    let B = W - S;
    let P = num_pi_symbols(source_block_symbols) as usize;
    let L = num_intermediate_symbols(source_block_symbols) as usize;

    let mut matrix = OctetMatrix::new(L, L);

    // G_LDPC,1
    // See section 5.3.3.3
    for i in 0..B {
        let a = 1 + i / S;

        let b = i % S;
        matrix.set(b, i, 1.into());

        let b = (b + a) % S;
        matrix.set(b, i, 1.into());

        let b = (b + a) % S;
        matrix.set(b, i, 1.into());
    }

    // I_S
    for i in 0..S {
        matrix.set(i as usize, i + B as usize, 1.into());
    }

    // G_LDPC,2
    // See section 5.3.3.3
    for i in 0..S {
        matrix.set(i, (i % P) + W, 1.into());
        matrix.set(i, ((i + 1) % P) + W, 1.into());
    }

    // G_HDPC
    let g_hdpc = generate_mt(H, Kprime, S) * generate_gamma(Kprime, S);
    for i in 0..H {
        for j in 0..(Kprime + S) {
            matrix.set(i + S, j, g_hdpc.get(i, j));
        }
    }

    // I_H
    for i in 0..H {
        matrix.set(i + S as usize, i + (Kprime + S) as usize, 1.into());
    }

    // G_ENC
    let mut row = 0;
    for i in encoded_symbol_indices {
        // row != i, because i is the ESI
        let tuple = intermediate_tuple(Kprime as u32, i);

        for j in enc_indices(Kprime as u32, tuple) {
            matrix.set(row as usize + S + H, j, 1.into());
        }
        row += 1;
    }

    matrix
}
