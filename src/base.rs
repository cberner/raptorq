use std::cmp::min;

use crate::rng::rand;
use crate::systematic_constants::calculate_p1;
use crate::systematic_constants::num_lt_symbols;
use crate::systematic_constants::systematic_index;
use crate::systematic_constants::SYSTEMATIC_INDICES_AND_PARAMETERS;

// As defined in section 3.2
#[derive(Clone)]
pub struct PayloadId {
    source_block_number: u8,
    encoding_symbol_id: u32
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

    pub fn source_block_number(&self) -> u8 {
        self.source_block_number
    }

    pub fn encoding_symbol_id(&self) -> u32 {
        self.encoding_symbol_id
    }
}

// As defined in section 4.4.2
#[derive(Clone)]
pub struct EncodingPacket {
    payload_id: PayloadId,
    data: Vec<u8>
}

impl EncodingPacket {
    pub fn new(payload_id: PayloadId, data: Vec<u8>) -> EncodingPacket {
        EncodingPacket {
            payload_id,
            data
        }
    }

    pub fn payload_id(&self) -> PayloadId {
        self.payload_id.clone()
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

// As defined in section 3.3.2 and 3.3.3
#[derive(Clone)]
pub struct ObjectTransmissionInformation {
    transfer_length: u64, // Limited to u40
    symbol_size: u16,
    num_source_blocks: u8,
    num_sub_blocks: u16,
    symbol_alignment: u8
}

impl ObjectTransmissionInformation {
    pub fn new(transfer_length: u64, symbol_size: u16, source_blocks: u8, sub_blocks: u16, alignment: u8) -> ObjectTransmissionInformation {
        assert!(transfer_length <= 946270874880);
        assert_eq!(symbol_size % alignment as u16, 0);
        ObjectTransmissionInformation {
            transfer_length,
            symbol_size,
            num_source_blocks: source_blocks,
            num_sub_blocks: sub_blocks,
            symbol_alignment: alignment
        }
    }

    pub fn transfer_length(&self) -> u64 {
        self.transfer_length
    }

    pub fn symbol_size(&self) -> u16 {
        self.symbol_size
    }

    pub fn source_blocks(&self) -> u8 {
        self.num_source_blocks
    }

    pub fn sub_blocks(&self) -> u16 {
        self.num_sub_blocks
    }

    pub fn alignment(&self) -> u8 {
        self.symbol_alignment
    }

    pub fn with_defaults(transfer_length: u64, max_packet_size: u16) -> ObjectTransmissionInformation {
        let alignment = 8;
        assert!(max_packet_size >= alignment);
        let symbol_size = max_packet_size - (max_packet_size % alignment);
        let max_memory = 10*1024*1024;
        let sub_symbol_size = 8;

        let kt = (transfer_length as f64 / symbol_size as f64).ceil();
        let n_max = (symbol_size as f64 / (sub_symbol_size * alignment) as f64).floor() as u32;

        let kl = |n: u32| -> u32 {
            for &(kprime, _, _, _, _) in SYSTEMATIC_INDICES_AND_PARAMETERS.iter().rev() {
                let x = (symbol_size as f64 / (alignment as u32 * n) as f64).ceil();
                if kprime <= (max_memory as f64 / (alignment as f64 * x)) as u32 {
                    return kprime;
                }
            }
            unreachable!();
        };

        let num_source_blocks = (kt / kl(n_max) as f64).ceil() as u32;

        let mut n = 1;
        for i in 1..=n_max {
            n = i;
            if (kt / num_source_blocks as f64).ceil() as u32 <= kl(n) {
                break;
            }
        }

        ObjectTransmissionInformation {
            transfer_length,
            symbol_size,
            num_source_blocks: num_source_blocks as u8,
            num_sub_blocks: n as u16,
            symbol_alignment: alignment as u8

        }
    }
}

// Partition[I, J] function, as defined in section 4.4.1.2
pub fn partition(i: u32, j: u32) -> (u32, u32, u32, u32) {
    let il = (i as f64 / j as f64).ceil() as u32;
    let is = (i as f64 / j as f64).floor() as u32;
    let jl = i - is * j;
    let js = j - jl;
    (il, is, jl, js)
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
    unreachable!();
}

// Tuple[K', X] as defined in section 5.3.5.4
#[allow(non_snake_case)]
pub fn intermediate_tuple(source_block_symbols: u32, internal_symbol_id: u32) -> (u32, u32, u32, u32, u32, u32) {
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
        d1 = 2 + rand(internal_symbol_id, 3, 2);
    }

    let a1 = 1 + rand(internal_symbol_id, 4, P1-1);
    let b1 = rand(internal_symbol_id, 5, P1);

    (d, a, b, d1, a1, b1)
}
