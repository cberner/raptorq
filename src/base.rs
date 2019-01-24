// As defined in section 3.2
pub struct PayloadId {
    pub source_block_number: u8,
    pub encoding_symbol_id: u32
}

impl PayloadId {
    pub fn new(source_block_number: u8, encoding_symbol_id: u32) -> Option<PayloadId> {
        // Encoding Symbol ID must be a 24-bit unsigned int
        if encoding_symbol_id >= 16777216 {
            return None
        }
        Some(PayloadId {
            source_block_number,
            encoding_symbol_id
        })
    }
}

// As defined in section 4.4.2
pub struct EncodingPacket {
    pub payload_id: PayloadId,
    pub symbol: Vec<u8>
}

