use std::collections::HashSet;
use base::EncodingPacket;
use symbol::Symbol;
use systematic_constants::extended_source_block_symbols;
use systematic_constants::num_ldpc_symbols;
use systematic_constants::num_hdpc_symbols;
use constraint_matrix::generate_constraint_matrix;
use base::intermediate_tuple;
use pi_solver::fused_inverse_mul_symbols;
use constraint_matrix::enc_indices;
use ::{ObjectTransmissionInformation, Octet};
use base::partition;

pub struct Decoder {
    config: ObjectTransmissionInformation,
    blocks: Vec<SourceBlockDecoder>
}

impl Decoder {
    pub fn new(config: ObjectTransmissionInformation) -> Decoder {
        Octet::static_init();
        let kt = (config.transfer_length() as f64 / config.symbol_size() as f64).ceil() as u32;
        let (kl, ks, zl, zs) = partition(kt, config.source_blocks() as u32);

        // TODO: support subblocks
        assert_eq!(1, config.sub_blocks());
//        let (tl, ts, nl, ns) = partition((config.symbol_size() / config.alignment() as u16) as u32, config.sub_blocks() as u32);

        let mut blocks = vec![];
        for i in 0..zl {
            blocks.push(SourceBlockDecoder::new(i as u8, config.symbol_size(), kl as u64 * config.symbol_size() as u64));
        }

        for i in 0..zs {
            blocks.push(SourceBlockDecoder::new(i as u8, config.symbol_size(), ks as u64 * config.symbol_size() as u64));
        }

        Decoder {
            config,
            blocks
        }
    }

    pub fn decode(&mut self, packet: EncodingPacket) -> Option<Vec<u8>> {
        self.blocks[packet.payload_id().source_block_number as usize].parse(packet);
        for block in self.blocks.iter() {
            if !block.is_decoded() {
                return None;
            }
        }

        let mut result = vec![];
        for block in self.blocks.iter_mut() {
            result.extend(block.get_data().unwrap());
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
    decoded: bool
}

impl SourceBlockDecoder {
    pub fn new(source_block_id: u8, symbol_size: u16, block_length: u64) -> SourceBlockDecoder {
        Octet::static_init();
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
            decoded: false
        }
    }

    pub fn is_decoded(&self) -> bool {
        self.decoded
    }

    // TODO: it would be nice if this function wasn't mut
    pub fn get_data(&mut self) -> Option<Vec<u8>> {
        let num_extended_symbols = extended_source_block_symbols(self.source_block_symbols);
        if self.received_source_symbols == self.source_block_symbols {
            let mut result = vec![];
            for symbol in self.source_symbols.clone() {
                result.extend(symbol.unwrap().bytes());
            }

            self.decoded = true;
            return Some(result);
        }

        if self.received_esi.len() as u32 >= num_extended_symbols {
            let s = num_ldpc_symbols(self.source_block_symbols) as usize;
            let h = num_hdpc_symbols(self.source_block_symbols) as usize;

            let mut encoded_indices = vec![];
            // See section 5.3.3.4.2. There are S + H zero symbols to start the D vector
            let mut d = vec![Symbol::zero(self.symbol_size as usize); s + h];
            for i in 0..self.source_symbols.len() {
                let symbol = self.source_symbols[i].clone();
                if symbol != None {
                    encoded_indices.push(i as u32);
                    d.push(symbol.unwrap());
                }
            }

            // Append the extended padding symbols
            for i in self.source_block_symbols..num_extended_symbols {
                encoded_indices.push(i);
                d.push(Symbol::zero(self.symbol_size as usize));
            }

            for repair_packet in self.repair_packets.iter() {
                encoded_indices.push(repair_packet.payload_id().encoding_symbol_id);
                d.push(Symbol::new(repair_packet.data().clone()));
            }

            let constraint_matrix = generate_constraint_matrix(self.source_block_symbols, &encoded_indices);
            let intermediate_symbols =  fused_inverse_mul_symbols(constraint_matrix, d, self.source_block_symbols);

            if intermediate_symbols == None {
                return None
            }
            let intermediate_symbols = intermediate_symbols.unwrap();

            let mut result = vec![];
            for i in 0..self.source_block_symbols as usize {
                if self.source_symbols[i] != None {
                    result.extend(self.source_symbols[i].clone().unwrap().bytes())
                }
                else {
                    let rebuilt = self.rebuild_source_symbol(&intermediate_symbols, i as u32);
                    result.extend(rebuilt.bytes());
                }
            }

            self.decoded = true;
            return Some(result);
        }
        None
    }

    pub fn parse(& mut self, packet: EncodingPacket) -> Option<Vec<u8>> {
        assert_eq!(self.source_block_id, packet.payload_id().source_block_number);
        let num_extended_symbols = extended_source_block_symbols(self.source_block_symbols);
        if self.received_esi.insert(packet.payload_id().encoding_symbol_id) {
            if packet.payload_id().encoding_symbol_id >= num_extended_symbols {
                // Repair symbol
                self.repair_packets.push(packet);
            }
            else {
                // Check that this is not an extended symbol (which aren't explicitly sent)
                assert!(packet.payload_id().encoding_symbol_id < self.source_block_symbols);
                // Source symbol
                self.source_symbols[packet.payload_id().encoding_symbol_id as usize] = Some(Symbol::new(packet.data().clone()));
                self.received_source_symbols += 1;
            }
        }

        self.get_data()
    }

    fn rebuild_source_symbol(&self, intermediate_symbols: &Vec<Symbol>, source_symbol_id: u32) -> Symbol {
        let tuple = intermediate_tuple(self.source_block_symbols, source_symbol_id);

        let mut rebuilt = Symbol::zero(self.symbol_size as usize);
        for i in enc_indices(self.source_block_symbols, tuple) {
            rebuilt += &intermediate_symbols[i];
        }
        rebuilt
    }
}

