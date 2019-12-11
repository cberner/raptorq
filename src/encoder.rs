use crate::base::intermediate_tuple;
use crate::base::partition;
use crate::base::EncodingPacket;
use crate::base::PayloadId;
use crate::constraint_matrix::generate_constraint_matrix;
use crate::matrix::DenseBinaryMatrix;
use crate::operation_vector::{perform_op, SymbolOps};
use crate::pi_solver::fused_inverse_mul_symbols;
use crate::sparse_matrix::SparseBinaryMatrix;
use crate::symbol::Symbol;
use crate::systematic_constants::extended_source_block_symbols;
use crate::systematic_constants::num_hdpc_symbols;
use crate::systematic_constants::num_intermediate_symbols;
use crate::systematic_constants::num_ldpc_symbols;
use crate::systematic_constants::num_lt_symbols;
use crate::systematic_constants::num_pi_symbols;
use crate::systematic_constants::{calculate_p1, systematic_index};
use crate::ObjectTransmissionInformation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub const SPARSE_MATRIX_THRESHOLD: u32 = 250;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Encoder {
    config: ObjectTransmissionInformation,
    blocks: Vec<SourceBlockEncoder>,
}

impl Encoder {
    pub fn with_defaults(data: &[u8], maximum_transmission_unit: u16) -> Encoder {
        let config = ObjectTransmissionInformation::with_defaults(
            data.len() as u64,
            maximum_transmission_unit,
        );

        let kt = (config.transfer_length() as f64 / config.symbol_size() as f64).ceil() as u32;
        let (kl, ks, zl, zs) = partition(kt, config.source_blocks());

        // TODO: support subblocks
        assert_eq!(1, config.sub_blocks());
        //        let (tl, ts, nl, ns) = partition((config.symbol_size() / config.alignment() as u16) as u32, config.sub_blocks());

        let cache = SourceBlockEncoderCache::new();
        let mut data_index = 0;
        let mut blocks = vec![];
        for i in 0..zl {
            let offset = kl as usize * config.symbol_size() as usize;
            blocks.push(SourceBlockEncoder::new(
                i as u8,
                config.symbol_size(),
                &data[data_index..(data_index + offset)],
                Some(&cache),
            ));
            data_index += offset;
        }

        for i in 0..zs {
            let offset = ks as usize * config.symbol_size() as usize;
            if data_index + offset <= data.len() {
                blocks.push(SourceBlockEncoder::new(
                    i as u8,
                    config.symbol_size(),
                    &data[data_index..(data_index + offset)],
                    Some(&cache),
                ));
            } else {
                // Should only be possible when Kt * T > F. See third to last paragraph in section 4.4.1.2
                assert!(kt as usize * config.symbol_size() as usize > data.len());
                // Zero pad the last symbol
                let mut padded = Vec::from(&data[data_index..]);
                padded.extend(vec![
                    0;
                    kt as usize * config.symbol_size() as usize - data.len()
                ]);
                blocks.push(SourceBlockEncoder::new(
                    i as u8,
                    config.symbol_size(),
                    &padded,
                    Some(&cache),
                ));
            }
            data_index += offset;
        }

        Encoder { config, blocks }
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

#[derive(Default)]
pub struct SourceBlockEncoderCache {
    cache: Arc<RwLock<HashMap<usize, Vec<SymbolOps>>>>,
}

impl SourceBlockEncoderCache {
    pub fn new() -> SourceBlockEncoderCache {
        let cache = Arc::new(RwLock::new(HashMap::new()));
        SourceBlockEncoderCache { cache }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceBlockEncoder {
    source_block_id: u8,
    source_symbols: Vec<Symbol>,
    intermediate_symbols: Vec<Symbol>,
}

impl SourceBlockEncoder {
    pub fn new(
        source_block_id: u8,
        symbol_size: u16,
        data: &[u8],
        cache: Option<&SourceBlockEncoderCache>,
    ) -> SourceBlockEncoder {
        assert_eq!(data.len() % symbol_size as usize, 0);
        let source_symbols: Vec<Symbol> = data
            .chunks(symbol_size as usize)
            .map(|x| Symbol::new(Vec::from(x)))
            .collect();

        let intermediate_symbols = match cache {
            Some(c) => {
                let key = source_symbols.len();
                let read_map = c.cache.read().unwrap();
                let value = read_map.get(&key);

                match value {
                    None => {
                        drop(read_map);
                        let (is, ops_vec) = gen_intermediate_symbols(
                            &source_symbols,
                            symbol_size as usize,
                            SPARSE_MATRIX_THRESHOLD,
                            true,
                        );
                        let mut write_map = c.cache.write().unwrap();
                        write_map.insert(key, ops_vec.unwrap());
                        drop(write_map);
                        is.unwrap()
                    }
                    Some(operation_vector) => {
                        let is = gen_intermediate_symbols_ops_vec(
                            &source_symbols,
                            symbol_size as usize,
                            &(*operation_vector),
                        );
                        drop(read_map);
                        is
                    }
                }
            }
            None => {
                let (is, _ops_vec) = gen_intermediate_symbols(
                    &source_symbols,
                    symbol_size as usize,
                    SPARSE_MATRIX_THRESHOLD,
                    false,
                );
                is.unwrap()
            }
        };

        SourceBlockEncoder {
            source_block_id,
            source_symbols,
            intermediate_symbols,
        }
    }

    pub fn source_packets(&self) -> Vec<EncodingPacket> {
        let mut esi: i32 = -1;
        self.source_symbols
            .iter()
            .map(|symbol| {
                esi += 1;
                EncodingPacket::new(
                    PayloadId::new(self.source_block_id, esi as u32),
                    symbol.as_bytes().to_vec(),
                )
            })
            .collect()
    }

    // See section 5.3.4
    pub fn repair_packets(&self, start_repair_symbol_id: u32, packets: u32) -> Vec<EncodingPacket> {
        let start_encoding_symbol_id = start_repair_symbol_id
            + extended_source_block_symbols(self.source_symbols.len() as u32);
        let mut result = vec![];
        let lt_symbols = num_lt_symbols(self.source_symbols.len() as u32);
        let sys_index = systematic_index(self.source_symbols.len() as u32);
        let p1 = calculate_p1(self.source_symbols.len() as u32);
        for i in 0..packets {
            let tuple = intermediate_tuple(start_encoding_symbol_id + i, lt_symbols, sys_index, p1);
            result.push(EncodingPacket::new(
                PayloadId::new(self.source_block_id, start_encoding_symbol_id + i),
                enc(
                    self.source_symbols.len() as u32,
                    &self.intermediate_symbols,
                    tuple,
                )
                .into_bytes(),
            ));
        }
        result
    }
}

#[allow(non_snake_case)]
fn create_d(
    source_block: &[Symbol],
    symbol_size: usize,
    extended_source_symbols: usize,
) -> Vec<Symbol> {
    let L = num_intermediate_symbols(source_block.len() as u32);
    let S = num_ldpc_symbols(source_block.len() as u32);
    let H = num_hdpc_symbols(source_block.len() as u32);

    let mut D = Vec::with_capacity(L as usize);
    for _ in 0..(S + H) {
        D.push(Symbol::zero(symbol_size));
    }
    for symbol in source_block {
        D.push(symbol.clone());
    }
    // Extend the source block with padding. See section 5.3.2
    for _ in 0..(extended_source_symbols as usize - source_block.len()) {
        D.push(Symbol::zero(symbol_size));
    }
    assert_eq!(D.len(), L as usize);
    D
}

// See section 5.3.3.4
#[allow(non_snake_case)]
fn gen_intermediate_symbols(
    source_block: &[Symbol],
    symbol_size: usize,
    sparse_threshold: u32,
    store_operations: bool,
) -> (Option<Vec<Symbol>>, Option<Vec<SymbolOps>>) {
    let extended_source_symbols = extended_source_block_symbols(source_block.len() as u32);
    let D = create_d(source_block, symbol_size, extended_source_symbols as usize);

    let indices: Vec<u32> = (0..extended_source_symbols).collect();
    if extended_source_symbols >= sparse_threshold {
        let (A, hdpc) =
            generate_constraint_matrix::<SparseBinaryMatrix>(extended_source_symbols, &indices);
        return fused_inverse_mul_symbols(A, hdpc, D, extended_source_symbols, store_operations);
    } else {
        let (A, hdpc) =
            generate_constraint_matrix::<DenseBinaryMatrix>(extended_source_symbols, &indices);
        return fused_inverse_mul_symbols(A, hdpc, D, extended_source_symbols, store_operations);
    }
}

#[allow(non_snake_case)]
fn gen_intermediate_symbols_ops_vec(
    source_block: &[Symbol],
    symbol_size: usize,
    operation_vector: &[SymbolOps],
) -> Vec<Symbol> {
    let extended_source_symbols = extended_source_block_symbols(source_block.len() as u32);
    let mut D = create_d(source_block, symbol_size, extended_source_symbols as usize);

    for op in operation_vector {
        perform_op(op, &mut D);
    }
    D
}

// Enc[] function, as defined in section 5.3.5.3
#[allow(clippy::many_single_char_names)]
fn enc(
    source_block_symbols: u32,
    intermediate_symbols: &[Symbol],
    source_tuple: (u32, u32, u32, u32, u32, u32),
) -> Symbol {
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

    use crate::base::intermediate_tuple;
    use crate::encoder::enc;
    use crate::encoder::gen_intermediate_symbols;
    use crate::symbol::Symbol;
    use crate::systematic_constants::num_lt_symbols;
    use crate::systematic_constants::num_pi_symbols;
    use crate::systematic_constants::{
        calculate_p1, num_ldpc_symbols, systematic_index, MAX_SOURCE_SYMBOLS_PER_BLOCK,
    };
    use crate::{Encoder, EncodingPacket};

    const SYMBOL_SIZE: usize = 4;
    const NUM_SYMBOLS: u32 = 100;

    fn gen_test_data(size: usize) -> Vec<u8> {
        let mut data: Vec<u8> = vec![0; size];
        for i in 0..size {
            data[i] = rand::thread_rng().gen();
        }
        data
    }

    fn gen_test_symbols() -> Vec<Symbol> {
        let mut source_block: Vec<Symbol> = vec![];
        for _ in 0..NUM_SYMBOLS {
            let data = gen_test_data(SYMBOL_SIZE);
            source_block.push(Symbol::new(data));
        }
        source_block
    }

    #[test]
    fn enc_constraint_dense() {
        enc_constraint(MAX_SOURCE_SYMBOLS_PER_BLOCK + 1);
    }

    #[test]
    fn enc_constraint_sparse() {
        enc_constraint(0);
    }

    fn enc_constraint(sparse_threshold: u32) {
        let source_symbols = gen_test_symbols();

        let (is, _ops_vec) =
            gen_intermediate_symbols(&source_symbols, SYMBOL_SIZE, sparse_threshold, false);
        let intermediate_symbols = is.unwrap();

        let lt_symbols = num_lt_symbols(NUM_SYMBOLS);
        let sys_index = systematic_index(NUM_SYMBOLS);
        let p1 = calculate_p1(NUM_SYMBOLS);
        // See section 5.3.3.4.1, item 1.
        for i in 0..source_symbols.len() {
            let tuple = intermediate_tuple(i as u32, lt_symbols, sys_index, p1);
            let encoded = enc(NUM_SYMBOLS, &intermediate_symbols, tuple);
            assert_eq!(source_symbols[i], encoded);
        }
    }

    #[test]
    fn ldpc_constraint_dense() {
        ldpc_constraint(MAX_SOURCE_SYMBOLS_PER_BLOCK + 1);
    }

    #[test]
    fn ldpc_constraint_sparse() {
        ldpc_constraint(0);
    }

    #[allow(non_snake_case)]
    fn ldpc_constraint(sparse_threshold: u32) {
        let (is, _ops_vec) =
            gen_intermediate_symbols(&gen_test_symbols(), SYMBOL_SIZE, sparse_threshold, false);
        let C = is.unwrap();
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

    #[test]
    fn padding_constraint_exact() {
        let packet_size: u16 = 1024;
        let padding_size: usize = 0;
        let data_size: usize = packet_size as usize * 2 - padding_size;
        padding_constraint(packet_size, padding_size, data_size);
    }

    #[test]
    fn padding_constraint_42_bytes() {
        let packet_size: u16 = 1024;
        let padding_size: usize = 42;
        let data_size: usize = packet_size as usize * 2 - padding_size;
        padding_constraint(packet_size, padding_size, data_size);
    }

    fn padding_constraint(packet_size: u16, padding_size: usize, data_size: usize) {
        let data = gen_test_data(data_size);
        let encoder = Encoder::with_defaults(&data, packet_size);

        fn accumulate_data(acc: Vec<u8>, packet: EncodingPacket) -> Vec<u8> {
            let mut updated_acc = acc.clone();
            updated_acc.extend_from_slice(packet.data());
            updated_acc
        }

        let padded_data = encoder
            .get_block_encoders()
            .iter()
            .flat_map(|block| block.source_packets())
            .fold(vec![], accumulate_data);

        assert_eq!(data_size + padding_size, padded_data.len());
        assert_eq!(data[..], padded_data[..data_size]);
    }
}
