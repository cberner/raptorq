use constants::SYSTEMATIC_INDICES_AND_PARAMETERS;

// K'_max as defined in section 5.1.2
const MAX_SOURCE_SYMBOLS_PER_BLOCK: u32 = 56403;

// Calculates, K', the extended source block size, in symbols, for a given source block size
// See section 5.3.1
pub fn extended_source_block_symbols(source_block_symbols: u32) -> u32 {
    assert!(source_block_symbols <= MAX_SOURCE_SYMBOLS_PER_BLOCK);
    for &(block_size, _, _, _, _) in SYSTEMATIC_INDICES_AND_PARAMETERS.iter() {
        if block_size >= source_block_symbols {
            return block_size;
        }
    }
    panic!(); // unreachable
}

// Calculates, J(K'), the systematic index, for a given number of source block symbols
// See section 5.6
pub fn systematic_index(source_block_symbols: u32) -> u32 {
    assert!(source_block_symbols <= MAX_SOURCE_SYMBOLS_PER_BLOCK);
    for &(block_size, systematic_index, _, _, _) in SYSTEMATIC_INDICES_AND_PARAMETERS.iter() {
        if block_size >= source_block_symbols {
            return systematic_index;
        }
    }
    panic!(); // unreachable
}

// Calculates, H(K'), the number of HDPC symbols, for a given number of source block symbols
// See section 5.6
pub fn num_hdpc_symbols(source_block_symbols: u32) -> u32 {
    assert!(source_block_symbols <= MAX_SOURCE_SYMBOLS_PER_BLOCK);
    for &(block_size, _, _, hdpc_symbols, _) in SYSTEMATIC_INDICES_AND_PARAMETERS.iter() {
        if block_size >= source_block_symbols {
            return hdpc_symbols;
        }
    }
    panic!(); // unreachable
}

// Calculates, S(K'), the number of LDPC symbols, for a given number of source block symbols
// See section 5.6
pub fn num_ldpc_symbols(source_block_symbols: u32) -> u32 {
    assert!(source_block_symbols <= MAX_SOURCE_SYMBOLS_PER_BLOCK);
    for &(block_size, _, ldpc_symbols, _, _) in SYSTEMATIC_INDICES_AND_PARAMETERS.iter() {
        if block_size >= source_block_symbols {
            return ldpc_symbols;
        }
    }
    panic!(); // unreachable
}

// Calculates, W(K'), the number of LT symbols, for a given number of source block symbols
// See section 5.6
pub fn num_lt_symbols(source_block_symbols: u32) -> u32 {
    assert!(source_block_symbols <= MAX_SOURCE_SYMBOLS_PER_BLOCK);
    for &(block_size, _, _, _, lt_symbols) in SYSTEMATIC_INDICES_AND_PARAMETERS.iter() {
        if block_size >= source_block_symbols {
            return lt_symbols;
        }
    }
    panic!(); // unreachable
}

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

#[cfg(test)]
mod tests {
    use base::MAX_SOURCE_SYMBOLS_PER_BLOCK;
    use base::num_ldpc_symbols;
    use base::num_lt_symbols;

    #[test]
    fn all_prime() {
        for i in 0..=MAX_SOURCE_SYMBOLS_PER_BLOCK {
            // See section 5.6
            assert!(primal::is_prime(num_ldpc_symbols(i) as u64));
            assert!(primal::is_prime(num_lt_symbols(i) as u64));
        }
    }
}
