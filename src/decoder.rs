use crate::base::intermediate_tuple;
use crate::base::partition;
use crate::base::EncodingPacket;
use crate::base::ObjectTransmissionInformation;
use crate::constraint_matrix::enc_indices;
use crate::constraint_matrix::generate_constraint_matrix;
use crate::pi_solver::fused_inverse_mul_symbols;
use crate::symbol::Symbol;
use crate::systematic_constants::extended_source_block_symbols;
use crate::systematic_constants::num_hdpc_symbols;
use crate::systematic_constants::num_ldpc_symbols;
use std::collections::HashSet;

pub struct Decoder {
    config: ObjectTransmissionInformation,
    block_decoders: Vec<SourceBlockDecoder>,
    blocks: Vec<Option<Vec<u8>>>,
}

impl Decoder {
    pub fn new(config: ObjectTransmissionInformation) -> Decoder {
        let kt = (config.transfer_length() as f64 / config.symbol_size() as f64).ceil() as u32;
        let (kl, ks, zl, zs) = partition(kt, config.source_blocks());

        // TODO: support subblocks
        assert_eq!(1, config.sub_blocks());
        //        let (tl, ts, nl, ns) = partition((config.symbol_size() / config.alignment() as u16) as u32, config.sub_blocks());

        let mut decoders = vec![];
        for i in 0..zl {
            decoders.push(SourceBlockDecoder::new(
                i as u8,
                config.symbol_size(),
                u64::from(kl) * u64::from(config.symbol_size()),
            ));
        }

        for i in 0..zs {
            decoders.push(SourceBlockDecoder::new(
                i as u8,
                config.symbol_size(),
                u64::from(ks) * u64::from(config.symbol_size()),
            ));
        }

        Decoder {
            config,
            block_decoders: decoders,
            blocks: vec![None; (zl + zs) as usize],
        }
    }

    pub fn decode(&mut self, packet: EncodingPacket) -> Option<Vec<u8>> {
        let block_number = packet.payload_id.source_block_number() as usize;
        if self.blocks[block_number].is_none() {
            self.blocks[block_number] = self.block_decoders[block_number].decode(packet);
        }
        for block in self.blocks.iter() {
            if block.is_none() {
                return None;
            }
        }

        let mut result = vec![];
        for block in self.blocks.iter() {
            result.extend(block.clone().unwrap());
        }
        result.truncate(self.config.transfer_length() as usize);
        Some(result)
    }
}

pub struct SourceBlockDecoder {
    source_block_id: u8,
    symbol_size: u16,
    source_block_symbols: u32,
    source_symbols: Vec<Option<Symbol>>,
    repair_packets: Vec<EncodingPacket>,
    received_source_symbols: u32,
    received_esi: HashSet<u32>,
    decoded: bool,
}

impl SourceBlockDecoder {
    pub fn new(source_block_id: u8, symbol_size: u16, block_length: u64) -> SourceBlockDecoder {
        let source_symbols = (block_length as f64 / symbol_size as f64).ceil() as u32;
        let mut received_esi = HashSet::new();
        for i in source_symbols..extended_source_block_symbols(source_symbols) {
            received_esi.insert(i);
        }
        SourceBlockDecoder {
            source_block_id,
            symbol_size,
            source_block_symbols: source_symbols,
            source_symbols: vec![None; source_symbols as usize],
            repair_packets: vec![],
            received_source_symbols: 0,
            received_esi,
            decoded: false,
        }
    }

    pub fn decode(&mut self, packet: EncodingPacket) -> Option<Vec<u8>> {
        assert_eq!(
            self.source_block_id,
            packet.payload_id.source_block_number()
        );

        let (payload_id, payload) = packet.split();
        let num_extended_symbols = extended_source_block_symbols(self.source_block_symbols);
        if self.received_esi.insert(payload_id.encoding_symbol_id()) {
            if payload_id.encoding_symbol_id() >= num_extended_symbols {
                // Repair symbol
                self.repair_packets
                    .push(EncodingPacket::new(payload_id, payload));
            } else {
                // Check that this is not an extended symbol (which aren't explicitly sent)
                assert!(payload_id.encoding_symbol_id() < self.source_block_symbols);
                // Source symbol
                self.source_symbols[payload_id.encoding_symbol_id() as usize] =
                    Some(Symbol::new(payload));
                self.received_source_symbols += 1;
            }
        }

        let num_extended_symbols = extended_source_block_symbols(self.source_block_symbols);
        if self.received_source_symbols == self.source_block_symbols {
            let result = self
                .source_symbols
                .iter()
                .cloned()
                .map(|symbol| symbol.unwrap().into_bytes())
                .flatten()
                .collect();

            self.decoded = true;
            return Some(result);
        }

        if self.received_esi.len() as u32 >= num_extended_symbols {
            let s = num_ldpc_symbols(self.source_block_symbols) as usize;
            let h = num_hdpc_symbols(self.source_block_symbols) as usize;

            let mut encoded_indices = vec![];
            // See section 5.3.3.4.2. There are S + H zero symbols to start the D vector
            let mut d = vec![Symbol::zero(self.symbol_size); s + h];
            for (i, source) in self.source_symbols.iter().enumerate() {
                if let Some(symbol) = source {
                    encoded_indices.push(i as u32);
                    d.push(symbol.clone());
                }
            }

            // Append the extended padding symbols
            for i in self.source_block_symbols..num_extended_symbols {
                encoded_indices.push(i);
                d.push(Symbol::zero(self.symbol_size));
            }

            for repair_packet in self.repair_packets.iter() {
                encoded_indices.push(repair_packet.payload_id.encoding_symbol_id());
                d.push(Symbol::new(repair_packet.data.clone()));
            }

            let constraint_matrix =
                generate_constraint_matrix(self.source_block_symbols, &encoded_indices);
            let intermediate_symbols =
                match fused_inverse_mul_symbols(constraint_matrix, d, self.source_block_symbols) {
                    None => return None,
                    Some(s) => s,
                };

            let mut result = vec![];
            for i in 0..self.source_block_symbols as usize {
                if let Some(ref symbol) = self.source_symbols[i] {
                    result.extend(symbol.as_bytes())
                } else {
                    let rebuilt = self.rebuild_source_symbol(&intermediate_symbols, i as u32);
                    result.extend(rebuilt.as_bytes());
                }
            }

            self.decoded = true;
            return Some(result);
        }
        None
    }

    fn rebuild_source_symbol(
        &self,
        intermediate_symbols: &[Symbol],
        source_symbol_id: u32,
    ) -> Symbol {
        let tuple = intermediate_tuple(self.source_block_symbols, source_symbol_id);

        let mut rebuilt = Symbol::zero(self.symbol_size);
        for i in enc_indices(self.source_block_symbols, tuple) {
            rebuilt += &intermediate_symbols[i];
        }
        rebuilt
    }
}
