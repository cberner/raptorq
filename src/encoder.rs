use base::EncodingPacket;
use PayloadId;
use systematic_constants::extended_source_block_symbols;
use systematic_constants::num_pi_symbols;
use symbol::Symbol;
use systematic_constants::num_lt_symbols;
use systematic_constants::calculate_p1;
use systematic_constants::num_intermediate_symbols;
use systematic_constants::num_ldpc_symbols;
use systematic_constants::num_hdpc_symbols;
use constraint_matrix::generate_constraint_matrix;
use base::intermediate_tuple;

pub struct SourceBlockEncoder {
    source_block_id: u8,
    symbol_size: u16,
    source_symbols: Vec<Symbol>,
    intermediate_symbols: Vec<Symbol>
}

impl SourceBlockEncoder {
    pub fn new(source_block_id: u8, symbol_size: u16, data: Vec<u8>) -> SourceBlockEncoder {
        assert_eq!(data.len() % symbol_size as usize, 0);
        let source_symbols: Vec<Symbol> = data.chunks(symbol_size as usize)
            .map(|x| Symbol::new(Vec::from(x)))
            .collect();
        let intermediate_symbols = gen_intermediate_symbols(extend_source_block(source_symbols.clone()));
        SourceBlockEncoder {
            source_block_id,
            symbol_size,
            source_symbols,
            intermediate_symbols
        }
    }

    pub fn all_source_packets(&self) -> Vec<EncodingPacket> {
        let mut esi: i32 = -1;
        self.source_symbols.iter().map(|symbol| {
            esi += 1;
            EncodingPacket {
                payload_id: PayloadId::new(self.source_block_id, esi as u32).unwrap(),
                symbol: symbol.clone()
            }
        }).collect()
    }

    // See section 5.3.4
    pub fn repair_packets(&self, start_repair_symbol_id: u32, packets: u32) -> Vec<EncodingPacket> {
        let start_encoding_symbol_id = start_repair_symbol_id + extended_source_block_symbols(self.source_symbols.len() as u32);
        let mut result = vec![];
        for i in 0..packets {
            let tuple = intermediate_tuple(self.source_symbols.len() as u32, start_encoding_symbol_id + i);
            result.push(EncodingPacket {
                payload_id: PayloadId::new(self.source_block_id, start_encoding_symbol_id + i).unwrap(),
                symbol: enc(self.source_symbols.len() as u32, self.intermediate_symbols.clone(), tuple)
            });
        }
        result
    }
}

// Extend the source block with padding. See section 5.3.2
fn extend_source_block(mut source_block: Vec<Symbol>) -> Vec<Symbol> {
    assert_ne!(0, source_block.len());
    let symbols = source_block.len() as u32;
    let symbol_size = source_block[0].value.len();
    let extended_source_symbols = extended_source_block_symbols(source_block.len() as u32);
    for _ in 0..(extended_source_symbols - symbols) {
        source_block.push(Symbol {
            value: vec![0; symbol_size]
        });
    }
    source_block
}

// See section 5.3.3.4
#[allow(non_snake_case)]
fn gen_intermediate_symbols(extended_source_block: Vec<Symbol>) -> Vec<Symbol> {
    let L = num_intermediate_symbols(extended_source_block.len() as u32);
    let S = num_ldpc_symbols(extended_source_block.len() as u32);
    let H = num_hdpc_symbols(extended_source_block.len() as u32);

    let mut D = vec![Symbol::zero(extended_source_block[0].value.len()); L as usize];
    for i in 0..extended_source_block.len() {
        D[(S + H) as usize + i] = extended_source_block[i].clone();
    }

    let A = generate_constraint_matrix(extended_source_block.len() as u32, 0..extended_source_block.len() as u32);
    A.inverse().unwrap().mul_symbols(&D)
}

// Enc[] function, as defined in section 5.3.5.3
fn enc(source_block_symbols: u32,
       intermediate_symbols: Vec<Symbol>,
       source_tuple: (u32, u32, u32, u32, u32, u32)) -> Symbol {
    let w = num_lt_symbols(source_block_symbols);
    let p = num_pi_symbols(source_block_symbols);
    let p1 = calculate_p1(source_block_symbols);
    let (d, a, mut b, d1, a1, mut b1) = source_tuple;

    assert!(1 <= a && a < w);
    assert!(b < w);
    assert!(d1 == 2 || d1 == 3);
    assert!(1 <= a1 && a < w);
    assert!(b1 < w);

    let mut result = intermediate_symbols[b as usize].clone();
    for _ in 1..d {
        b = (b + a) % w;
        result = result + intermediate_symbols[b as usize].clone();
    }

    while b1 >= p {
        b1 = (b1 + a1) % p1;
    }

    result = result + intermediate_symbols[(w + b1) as usize].clone();

    for _ in 1..d1 {
        b1 = (b1 + a1) % p1;
        while b1 >= p {
            b1 = (b1 + a1) % p1;
        }
        result = result + intermediate_symbols[(w + b1) as usize].clone();
    }

    result
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use symbol::Symbol;
    use encoder::tests::rand::Rng;
    use encoder::extend_source_block;
    use encoder::enc;
    use base::intermediate_tuple;
    use encoder::gen_intermediate_symbols;
    use systematic_constants::num_ldpc_symbols;
    use systematic_constants::num_lt_symbols;
    use systematic_constants::num_pi_symbols;

    const SYMBOL_SIZE: usize = 4;
    const NUM_SYMBOLS: u32 = 100;

    fn gen_test_symbols() -> Vec<Symbol> {
        let mut source_block: Vec<Symbol> = vec![];
        for _ in 0..NUM_SYMBOLS {
            let mut data: Vec<u8> = vec![0; SYMBOL_SIZE];
            for i in 0..SYMBOL_SIZE {
                data[i] = rand::thread_rng().gen();
            }
            source_block.push(Symbol {
                value: data
            });
        }

        extend_source_block(source_block)
    }

    #[test]
    fn enc_constraint() {
        let extended_source_symbols = gen_test_symbols();
        let intermediate_symbols = gen_intermediate_symbols(extended_source_symbols.clone());

        // See section 5.3.3.4.1, item 1.
        for i in 0..extended_source_symbols.len() {
            let tuple = intermediate_tuple(NUM_SYMBOLS, i as u32);
            let encoded = enc(NUM_SYMBOLS, intermediate_symbols.clone(), tuple);
            assert_eq!(extended_source_symbols[i].clone(), encoded);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn ldpc_constraint() {
        let C = gen_intermediate_symbols(gen_test_symbols());
        let S = num_ldpc_symbols(NUM_SYMBOLS) as usize;
        let P = num_pi_symbols(NUM_SYMBOLS) as usize;
        let W = num_lt_symbols(NUM_SYMBOLS) as usize;
        let B = W - S;

        // See section 5.3.3.3
        let mut D = vec![];
        for i in 0..S {
            D.push(C[B + i].clone());
        }

        for i in 0..B {
            let a = 1 + i / S;
            let b = i % S;
            D[b] = D[b].clone() + C[i].clone();

            let b = (b + a) % S;
            D[b] = D[b].clone() + C[i].clone();

            let b = (b + a) % S;
            D[b] = D[b].clone() + C[i].clone();
        }

        for i in 0..S {
            let a = i % P;
            let b = (i + 1) % P;
            D[i] = D[i].clone() + C[W + a].clone() + C[W + b].clone();
        }

        for i in 0..S {
            assert_eq!(Symbol::zero(SYMBOL_SIZE), D[i].clone());
        }
    }
}
