use std::cmp::min;
use systematic_constants::systematic_index;
use rng::rand;
use systematic_constants::num_intermediate_symbols;
use systematic_constants::num_lt_symbols;
use systematic_constants::num_pi_symbols;
use systematic_constants::calculate_p1;
use symbol::Symbol;
use matrix::OctetMatrix;
use std::collections::HashMap;
use octet::Octet;

// As defined in section 3.2
#[derive(Clone)]
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
#[derive(Clone)]
pub struct EncodingPacket {
    pub payload_id: PayloadId,
    pub symbol: Symbol
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

// See section 5.4.2.1
#[allow(non_snake_case)]
struct IntermediateSymbolDecoder {
    A: Vec<Vec<Octet>>,
    X: Vec<Vec<Octet>>,
    D: Vec<Symbol>,
    c: Vec<usize>,
    d: Vec<usize>,
    i: usize,
    L: usize,
    num_source_symbols: u32
}

impl IntermediateSymbolDecoder {
    fn new(matrix: &OctetMatrix, symbols: &Vec<Symbol>, num_source_symbols: u32) -> IntermediateSymbolDecoder {
        // TODO: implement for non-square matrices
        assert_eq!(matrix.width(), symbols.len());
        assert_eq!(matrix.height(), symbols.len());
        let mut c = vec![];
        let mut d = vec![];
        for i in 0..matrix.width() {
            c.push(i);
            d.push(i);
        }

        IntermediateSymbolDecoder {
            A: matrix.elements(),
            X: matrix.elements(),
            D: symbols.clone(),
            c,
            d,
            i: 0,
            L: num_intermediate_symbols(num_source_symbols) as usize,
            num_source_symbols
        }

    }

    // First phase (section 5.4.2.2)
    #[allow(non_snake_case)]
    fn first_phase(&mut self) -> bool {
        // First phase (section 5.4.2.2)
        let u = num_pi_symbols(self.num_source_symbols);

        true
    }

    // Second phase (section 5.4.2.3)
    #[allow(non_snake_case)]
    fn second_phase(&mut self) {

    }
    // Third phase (section 5.4.2.4)
    #[allow(non_snake_case)]
    fn third_phase(&mut self) {

    }

    // Fourth phase (section 5.4.2.5)
    #[allow(non_snake_case)]
    fn fourth_phase(&mut self) {

    }

    // Fifth phase (section 5.4.2.6)
    #[allow(non_snake_case)]
    fn fifth_phase(&mut self) {
        for j in 1..=self.i as usize {
            if self.A[j][j] != Octet::one() {
                let temp = self.A[j][j].clone();
                self.mul_row(j, Octet::one() / temp)
            }
            for l in 1..=j {
                let temp = self.A[j][l].clone();
                if temp != Octet::zero() {
                    self.fma_rows(j, l, temp);
                }
            }
        }
    }

    // Helper operations to apply operations to A, also to D
    fn mul_row(&mut self, i: usize, beta: Octet) {
        self.D[self.d[i]] = self.D[self.d[i]].mul_scalar(&beta);
        for j in 0..self.L {
            self.A[i][j] = &self.A[i][j] * &beta;
        }
    }

    fn fma_rows(&mut self, i: usize, iprime: usize, beta: Octet) {
        let temp = self.D[self.d[i]].clone();
        self.D[self.d[iprime]].fused_addassign_mul_scalar(&temp, &beta);
        for j in 0..self.L {
            self.A[iprime][j] += &self.A[i][j] * &beta;
        }
    }

    fn swap_rows(&mut self, i: usize, iprime: usize) {
        self.A.swap(i, iprime);
        self.d.swap(i, iprime);
    }

    fn swap_columns(&mut self, j: usize, jprime: usize) {
        for i in 0..self.A.len() {
            self.A[i].swap(j, jprime);
        }
        self.c.swap(j, jprime);
    }

    fn execute(&mut self) -> Option<Vec<Symbol>> {
        if !self.first_phase() {
            return None
        }
        self.second_phase();
        self.third_phase();
        self.fourth_phase();
        self.fifth_phase();

        // See end of section 5.4.2.1
        let mut index_mapping = HashMap::new();
        for i in 0..self.L {
            index_mapping.insert(self.c[i], self.d[i]);
        }
        let mut result = vec![];
        for i in 0..self.L {
            result.push(self.D[index_mapping[&i]].clone());
        }
        Some(result)
    }
}


// Fused implementation for self.inverse().mul_symbols(symbols)
// See section 5.4.2.1
pub fn fused_inverse_mul_symbols(matrix: &OctetMatrix, symbols: &Vec<Symbol>, num_source_symbols: u32) -> Option<Vec<Symbol>> {
    IntermediateSymbolDecoder::new(matrix, symbols, num_source_symbols).execute()
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use base::tests::rand::Rng;
    use symbol::Symbol;
    use matrix::OctetMatrix;
    use systematic_constants::extended_source_block_symbols;
    use constraint_matrix::generate_constraint_matrix;
    use base::fused_inverse_mul_symbols;

    fn identity(size: usize) -> OctetMatrix {
        let mut result = OctetMatrix::new(size, size);
        for i in 0..size {
            result.set(i, i, 1);
        }
        result
    }

    fn rand_symbol(symbol_size: usize) -> Symbol {
        let mut data: Vec<u8> = vec![0; symbol_size];
        for i in 0..symbol_size {
            data[i] = rand::thread_rng().gen();
        }
        Symbol::new(data)
    }

    #[test]
    #[ignore]
    fn inverse() {
        for &source_symbols in [5, 20, 30, 50, 100].iter() {
            let symbols = extended_source_block_symbols(source_symbols);
            let a = generate_constraint_matrix(source_symbols, 0..symbols);
            let identity = identity(a.height());
            assert_eq!(identity, a.clone() * a.inverse().unwrap());

            let mut rand_symbols = vec![];
            for _ in 0..a.width() {
                rand_symbols.push(rand_symbol(8));
            }
            assert_eq!(a.clone().inverse().unwrap().mul_symbols(&rand_symbols), fused_inverse_mul_symbols(&a, &rand_symbols, source_symbols).unwrap());
        }
    }
}
