use base::EncodingPacket;
use PayloadId;
use systematic_constants::extended_source_block_symbols;

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