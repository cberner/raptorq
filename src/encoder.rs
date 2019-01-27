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

pub struct SourceBlockEncoder {
    source_block_id: u8,
    symbol_size: u16,
    data: Vec<u8>
}

impl SourceBlockEncoder {
    pub fn new(source_block_id: u8, symbol_size: u16, data: Vec<u8>) -> SourceBlockEncoder {
        assert!(data.len() % symbol_size as usize == 0);
        SourceBlockEncoder {
            source_block_id,
            symbol_size,
            data
        }
    }

    fn num_source_symbols(&self) -> u32 {
        self.data.len() as u32 / self.symbol_size as u32
    }

    fn compute_intermediate_symbols(&self) {
        let extended_source_symbols = extended_source_block_symbols(self.num_source_symbols());
        // Extend the source block with padding. See section 5.3.2
        let mut extended_data = self.data.clone();
        for i in 0..((extended_source_symbols - self.num_source_symbols())*self.symbol_size as u32) {
            extended_data.push(0);
        }

    }

    pub fn all_source_packets(&self) -> Vec<EncodingPacket> {
        let mut esi: i32 = -1;
        self.data.chunks(self.symbol_size as usize)
            .map(|symbol| {
                esi += 1;
                EncodingPacket {
                    payload_id: PayloadId::new(self.source_block_id, esi as u32).unwrap(),
                    symbol: Vec::from(symbol)
                }
            }).collect()
    }

    pub fn repair_packets(&self, start_encoding_symbol_id: u32, packets: u32) -> Vec<EncodingPacket> {
        vec![]
    }
}

// Extend the source block with padding. See section 5.3.2
fn extend_source_block(mut source_block: Vec<Symbol>) -> Vec<Symbol> {
    assert_ne!(0, source_block.len());
    let symbols = source_block.len() as u32;
    let symbol_size = source_block[0].value.len();
    let extended_source_symbols = extended_source_block_symbols(source_block.len() as u32);
    for i in 0..(extended_source_symbols - symbols) {
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

    let A = generate_constraint_matrix(extended_source_block.len() as u32);
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
    for j in 1..d {
        b = (b + a) % w;
        result = result + intermediate_symbols[b as usize].clone();
    }

    while b1 >= p {
        b1 = (b1 + a1) % p1;
    }

    result = result + intermediate_symbols[(w + b1) as usize].clone();

    for j in 1..d1 {
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
    use systematic_constants::num_intermediate_symbols;
    use encoder::gen_intermediate_symbols;

    #[test]
    fn intermediate_symbol_definition() {
        let symbol_size = 4;
        let num_symbols = 100;
        let mut source_block: Vec<Symbol> = vec![];
        for _ in 0..num_symbols {
            let mut data: Vec<u8> = vec![0; symbol_size];
            for i in 0..symbol_size {
                data[i] = rand::thread_rng().gen();
            }
            source_block.push(Symbol {
                value: data
            });
        }

        let extended_source_block = extend_source_block(source_block);
        let intermediate_symbols = gen_intermediate_symbols(extended_source_block);

        // See section 5.3.3.4.1, item 1.
        for i in 0..intermediate_symbols.len() {
            let tuple = intermediate_tuple(num_symbols, i as u32);
            let encoded = enc(num_symbols, intermediate_symbols.clone(), tuple);
            assert_eq!(intermediate_symbols[i], encoded);
        }
    }
}
