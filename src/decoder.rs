use std::collections::HashSet;
use base::EncodingPacket;

pub struct SourceBlockDecoder {
    source_block_id: u8,
    symbol_size: u16,
    block_length: u64,
    data: Vec<u8>,
    received_esi: HashSet<u32>
}

impl SourceBlockDecoder {
    pub fn new(source_block_id: u8, symbol_size: u16, block_length: u64) -> SourceBlockDecoder {
        SourceBlockDecoder {
            source_block_id,
            symbol_size,
            block_length,
            data: vec![0; block_length as usize],
            received_esi: HashSet::new()
        }
    }

    pub fn parse(& mut self, packet: EncodingPacket) -> Option<Vec<u8>> {
        self.received_esi.insert(packet.payload_id.encoding_symbol_id);
        let x = (packet.payload_id.encoding_symbol_id * self.symbol_size as u32) as usize;
        for i in 0..packet.symbol.len() {
            self.data[x + i] = packet.symbol[i];
        }

        if (self.received_esi.len() * self.symbol_size as usize) as u64 >= self.block_length {
            return Some(self.data.clone());
        }
        None
    }
}