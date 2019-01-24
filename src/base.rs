use std::cmp::min;
use systematic_constants::num_intermediate_symbols;
use systematic_constants::systematic_index;
use rng::rand;
use systematic_constants::num_lt_symbols;
use systematic_constants::calculate_p1;

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

// Deg[v] as defined in section 5.3.5.2
pub fn deg(v: u32, lt_symbols: u32) -> u32 {
    assert!(v < 1048576);
    let f: [u32; 31] = [
        0, 5243, 529531, 704294, 791675, 844104, 879057, 904023, 922747, 937311, 948962,
        958494, 966438, 973160, 978921, 983914, 988283, 992138, 995565, 998631, 1001391,
        1003887, 1006157, 1008229, 1010129, 1011876, 1013490, 1014983, 1016370, 1017662, 1048576];

    for d in 1..f.len() {
        if v < f[d] {
            return min(d as u32, lt_symbols - 2);
        }
    }
    panic!();
}

// Tuple[K', X] as defined in section 5.3.5.4
#[allow(non_snake_case)]
pub fn intermediate_tuple(source_block_symbols: u32, internal_symbol_id: u32) -> (u32, u32, u32, u32, u32, u32) {
    let L = num_intermediate_symbols(source_block_symbols);
    let J = systematic_index(source_block_symbols);
    let W = num_lt_symbols(source_block_symbols);
    let P1 = calculate_p1(source_block_symbols);

    let mut A = 53591 + J*997;

    if A % 2 == 0 {
        A = A + 1
    }

    let B = 10267*(J + 1);
    let y: u32 = ((B as u64 + internal_symbol_id as u64 * A as u64) % 4294967296) as u32;
    let v = rand(y, 0, 1048576);
    let d = deg(v, W);
    let a = 1 + rand(y, 1, W-1);
    let b = rand(y, 2, W);

    let mut d1 = 2;
    if d < 4 {
        let d1 = 2 + rand(internal_symbol_id, 3, 2);
    }

    let a1 = 1 + rand(internal_symbol_id, 4, P1-1);
    let b1 = rand(internal_symbol_id, 5, P1);

    (d, a, b, d1, a1, b1)
}
