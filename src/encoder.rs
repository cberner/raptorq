use base::EncodingPacket;
use PayloadId;

pub struct SourceBlockEncoder {
    source_block_id: u8,
    symbol_size: u16,
    data: Vec<u8>
}

impl SourceBlockEncoder {
    pub fn new(source_block_id: u8, symbol_size: u16, data: Vec<u8>) -> SourceBlockEncoder {
        SourceBlockEncoder {
            source_block_id,
            symbol_size,
            data
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