use crate::base::EncodingPacket;
use crate::base::PayloadId;
use crate::systematic_constants::extended_source_block_symbols;
use crate::systematic_constants::num_pi_symbols;
use crate::symbol::Symbol;
use crate::systematic_constants::num_lt_symbols;
use crate::systematic_constants::calculate_p1;
use crate::systematic_constants::num_intermediate_symbols;
use crate::systematic_constants::num_ldpc_symbols;
use crate::systematic_constants::num_hdpc_symbols;
use crate::constraint_matrix::generate_constraint_matrix;
use crate::base::intermediate_tuple;
use crate::pi_solver::fused_inverse_mul_symbols;
use crate::base::partition;
use crate::ObjectTransmissionInformation;

pub struct Encoder {
    config: ObjectTransmissionInformation,
    blocks: Vec<SourceBlockEncoder>
}

impl Encoder {
    pub fn with_defaults(data: &[u8], maximum_transmission_unit: u16) -> Encoder {
        let config = ObjectTransmissionInformation::with_defaults(data.len() as u64, maximum_transmission_unit);

        let kt = (config.transfer_length() as f64 / config.symbol_size() as f64).ceil() as u32;
        let (kl, ks, zl, zs) = partition(kt, config.source_blocks() as u32);

        // TODO: support subblocks
        assert_eq!(1, config.sub_blocks());
//        let (tl, ts, nl, ns) = partition((config.symbol_size() / config.alignment() as u16) as u32, config.sub_blocks() as u32);

        let mut data_index = 0;
        let mut blocks = vec![];
        for i in 0..zl {
            let offset = kl as usize * config.symbol_size() as usize;
            blocks.push(SourceBlockEncoder::new(i as u8, config.symbol_size(), &data[data_index..(data_index + offset)]));
            data_index += offset;
        }

        for i in 0..zs {
            let offset = ks as usize * config.symbol_size() as usize;
            if data_index + offset < data.len() {
                blocks.push(SourceBlockEncoder::new(i as u8, config.symbol_size(), &data[data_index..(data_index + offset)]));
            }
            else {
                // Should only be possible when Kt * T > F. See third to last paragraph in section 4.4.1.2
                assert!(kt as usize * config.symbol_size() as usize > data.len());
                // Zero pad the last symbol
                let mut padded = Vec::from(&data[data_index..]);
                padded.extend(vec![0; kt as usize * config.symbol_size() as usize - data.len()]);
                blocks.push(SourceBlockEncoder::new(i as u8, config.symbol_size(), &padded));
            }
            data_index += offset;
        }

        Encoder {
            config,
            blocks
        }
    }

    pub fn get_config(&self) -> ObjectTransmissionInformation {
        self.config.clone()
    }

    pub fn get_encoded_packets(&self, repair_packets_per_block: u32) -> Vec<EncodingPacket> {
        let mut packets = vec![];
        for encoder in self.blocks.iter() {
            packets.extend(encoder.source_packets());
            packets.extend(encoder.repair_packets(0, repair_packets_per_block));
        }
        packets
    }

    pub fn get_block_encoders(&self) -> &Vec<SourceBlockEncoder> {
        &self.blocks
    }
}

pub struct SourceBlockEncoder {
    source_block_id: u8,
    source_symbols: Vec<Symbol>,
    intermediate_symbols: Vec<Symbol>
}

impl SourceBlockEncoder {
    pub fn new(source_block_id: u8, symbol_size: u16, data: &[u8]) -> SourceBlockEncoder {
        assert_eq!(data.len() % symbol_size as usize, 0);
        let source_symbols: Vec<Symbol> = data.chunks(symbol_size as usize)
            .map(|x| Symbol::new(Vec::from(x)))
            .collect();
        let intermediate_symbols = gen_intermediate_symbols(&source_symbols, symbol_size as usize);
        SourceBlockEncoder {
            source_block_id,
            source_symbols,
            intermediate_symbols
        }
    }

    pub fn source_packets(&self) -> Vec<EncodingPacket> {
        let mut esi: i32 = -1;
        self.source_symbols.iter().map(|symbol| {
            esi += 1;
            EncodingPacket::new(
                PayloadId::new(self.source_block_id, esi as u32).unwrap(),
                symbol.bytes().clone()
            )
        }).collect()
    }

    // See section 5.3.4
    pub fn repair_packets(&self, start_repair_symbol_id: u32, packets: u32) -> Vec<EncodingPacket> {
        let start_encoding_symbol_id = start_repair_symbol_id + extended_source_block_symbols(self.source_symbols.len() as u32);
        let mut result = vec![];
        for i in 0..packets {
            let tuple = intermediate_tuple(self.source_symbols.len() as u32, start_encoding_symbol_id + i);
            result.push(EncodingPacket::new(
                PayloadId::new(self.source_block_id, start_encoding_symbol_id + i).unwrap(),
                enc(self.source_symbols.len() as u32, &self.intermediate_symbols, tuple).bytes().clone()
            ));
        }
        result
    }
}

// See section 5.3.3.4
#[allow(non_snake_case)]
fn gen_intermediate_symbols(source_block: &Vec<Symbol>, symbol_size: usize) -> Vec<Symbol> {
    let L = num_intermediate_symbols(source_block.len() as u32);
    let S = num_ldpc_symbols(source_block.len() as u32);
    let H = num_hdpc_symbols(source_block.len() as u32);
    let extended_source_symbols = extended_source_block_symbols(source_block.len() as u32);

    let mut D = Vec::with_capacity(L as usize);
    for _ in 0..(S + H) {
        D.push(Symbol::zero(symbol_size));
    }
    for i in 0..source_block.len() {
        D.push(source_block[i].clone());
    }
    // Extend the source block with padding. See section 5.3.2
    for _ in 0..(extended_source_symbols as usize - source_block.len()) {
        D.push(Symbol::zero(symbol_size));
    }
    assert_eq!(D.len(), L as usize);

    let indices: Vec<u32> = (0..extended_source_symbols).collect();
    let A = generate_constraint_matrix(extended_source_symbols, &indices);
    fused_inverse_mul_symbols(A, D, extended_source_symbols).unwrap()
}

// Enc[] function, as defined in section 5.3.5.3
fn enc(source_block_symbols: u32,
       intermediate_symbols: &Vec<Symbol>,
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
        result += &intermediate_symbols[b as usize];
    }

    while b1 >= p {
        b1 = (b1 + a1) % p1;
    }

    result += &intermediate_symbols[(w + b1) as usize];

    for _ in 1..d1 {
        b1 = (b1 + a1) % p1;
        while b1 >= p {
            b1 = (b1 + a1) % p1;
        }
        result += &intermediate_symbols[(w + b1) as usize];
    }

    result
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::symbol::Symbol;
    use crate::encoder::enc;
    use crate::base::intermediate_tuple;
    use crate::encoder::gen_intermediate_symbols;
    use crate::systematic_constants::num_ldpc_symbols;
    use crate::systematic_constants::num_lt_symbols;
    use crate::systematic_constants::num_pi_symbols;

    const SYMBOL_SIZE: usize = 4;
    const NUM_SYMBOLS: u32 = 100;

    fn gen_test_symbols() -> Vec<Symbol> {
        let mut source_block: Vec<Symbol> = vec![];
        for _ in 0..NUM_SYMBOLS {
            let mut data: Vec<u8> = vec![0; SYMBOL_SIZE];
            for i in 0..SYMBOL_SIZE {
                data[i] = rand::thread_rng().gen();
            }
            source_block.push(Symbol::new(data));
        }
        source_block
    }

    #[test]
    fn enc_constraint() {
        let source_symbols = gen_test_symbols();
        let intermediate_symbols = gen_intermediate_symbols(&source_symbols, SYMBOL_SIZE);

        // See section 5.3.3.4.1, item 1.
        for i in 0..source_symbols.len() {
            let tuple = intermediate_tuple(NUM_SYMBOLS, i as u32);
            let encoded = enc(NUM_SYMBOLS, &intermediate_symbols, tuple);
            assert_eq!(source_symbols[i], encoded);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn ldpc_constraint() {
        let C = gen_intermediate_symbols(&gen_test_symbols(), SYMBOL_SIZE);
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
            D[b] += &C[i];

            let b = (b + a) % S;
            D[b] += &C[i];

            let b = (b + a) % S;
            D[b] += &C[i];
        }

        for i in 0..S {
            let a = i % P;
            let b = (i + 1) % P;
            D[i] += &C[W + a];
            D[i] += &C[W + b];
        }

        for i in 0..S {
            assert_eq!(Symbol::zero(SYMBOL_SIZE), D[i]);
        }
    }
}
