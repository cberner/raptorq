use std::cmp::min;
use systematic_constants::systematic_index;
use rng::rand;
use systematic_constants::num_intermediate_symbols;
use systematic_constants::num_lt_symbols;
use systematic_constants::num_pi_symbols;
use systematic_constants::num_ldpc_symbols;
use systematic_constants::num_hdpc_symbols;
use systematic_constants::calculate_p1;
use symbol::Symbol;
use matrix::OctetMatrix;
use std::collections::HashMap;
use octet::Octet;
use std::collections::HashSet;
use petgraph::prelude::*;
use petgraph::algo::condensation;

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
    u: usize,
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
            u: num_pi_symbols(num_source_symbols) as usize,
            L: num_intermediate_symbols(num_source_symbols) as usize,
            num_source_symbols
        }
    }

    // Returns true iff all elements in A between [start_row, end_row)
    // and [start_column, end_column) are zero
    fn all_zeroes(&self, start_row: usize, end_row: usize, start_column: usize, end_column: usize) -> bool {
        for row in start_row..end_row {
            for column in start_column..end_column {
                if self.A[row][column] != Octet::zero() {
                    return false;
                }
            }
        }
        return true;
    }

    // First phase (section 5.4.2.2)
    #[allow(non_snake_case)]
    fn first_phase(&mut self) -> bool {
        // First phase (section 5.4.2.2)

        //    ----------> i                 u <--------
        //  | +-----------+-----------------+---------+
        //  | |           |                 |         |
        //  | |     I     |    All Zeros    |         |
        //  v |           |                 |         |
        //  i +-----------+-----------------+    U    |
        //    |           |                 |         |
        //    |           |                 |         |
        //    | All Zeros |       V         |         |
        //    |           |                 |         |
        //    |           |                 |         |
        //    +-----------+-----------------+---------+
        // Figure 6: Submatrices of A in the First Phase

        let S = num_ldpc_symbols(self.num_source_symbols);
        let H = num_hdpc_symbols(self.num_source_symbols);

        let mut hdpc_rows = vec![false; self.A.len()];
        for row in S..(S + H) {
            hdpc_rows[row as usize] = true;
        }

        while self.i + self.u < self.L {
            if self.all_zeroes(self.i, self.L, self.i, self.L - self.u) {
                return false;
            }

            // Calculate r
            // "Let r be the minimum integer such that at least one row of A has
            // exactly r nonzeros in V."
            let mut r = self.L + 1;
            for row in self.i..self.L {
                let mut non_zero = 0;
                for col in self.i..(self.L - self.u) {
                    if self.A[row][col] != Octet::zero() {
                        non_zero += 1;
                    }
                }
                // Take the minimum positive integer, as the spec seems to be wrong about selecting
                // the minimum integer (if you do, the following code will fail in division by zero)
                if non_zero > 0 && non_zero < r {
                    r = non_zero;
                }
            }

            let mut chosen_row = None;
            if r == 2 {
                // See paragraph starting "If r = 2 and there is a row with exactly 2 ones in V..."
                let mut rows_with_two_ones = HashSet::new();
                for row in self.i..self.L {
                    let mut ones = 0;
                    for col in self.i..(self.L - self.u) {
                        if self.A[row][col] == Octet::one() {
                            ones += 1;
                        }
                        if ones > r {
                            break;
                        }
                    }
                    if ones == r {
                        rows_with_two_ones.insert(row);
                    }
                }

                if rows_with_two_ones.len() > 0 {
                    let mut g = Graph::new_undirected();
                    let mut node_lookup = HashMap::new();
                    for col in self.i..(self.L - self.u) {
                        let node = g.add_node(col);
                        node_lookup.insert(col, node);
                    }

                    for row in rows_with_two_ones.clone() {
                        if hdpc_rows[row] {
                            continue;
                        }
                        let mut ones = vec![];
                        for col in self.i..(self.L - self.u) {
                            // XXX: It's unclear exactly how to construct this graph. The RFC seems to have
                            // a typo. It says, emphasis mine, "The following graph defined by the structure
                            // of V is used in determining which row of A is chosen. The columns that
                            // intersect V are the nodes in the graph, and the rows that have *exactly 2
                            // nonzero* entries in V and are not HDPC rows are the edges of the graph that
                            // connect the two columns (nodes) in the positions of *the two ones*."
                            if self.A[row][col] == Octet::one() {
                                ones.push(col);
                            }
                            if ones.len() == 2 {
                                break;
                            }
                        }
                        let node1 = node_lookup[&ones[0]];
                        let node2 = node_lookup[&ones[1]];
                        g.add_edge(node1, node2, row);
                    }

                    let connected_components = condensation(g.clone(), true);
                    let mut row_to_component_size = HashMap::new();
                    for index in connected_components.node_indices() {
                        let cols = connected_components.node_weight(index).unwrap();
                        for col in cols {
                            for edge in g.edges(node_lookup[col]) {
                                row_to_component_size.insert(edge.weight(), cols.len());
                            }
                        }
                    }

                    let mut chosen_component_size = 0;
                    for row in rows_with_two_ones {
                        if hdpc_rows[row] {
                            continue;
                        }
                        if row_to_component_size[&row] > chosen_component_size {
                            chosen_row = Some(row);
                            chosen_component_size = row_to_component_size[&row];
                        }
                    }
                }
                else {
                    // See paragraph starting "If r = 2 and there is no row with exactly 2 ones in V"
                    for row in self.i..self.L {
                        let mut non_zero = 0;
                        for col in self.i..(self.L - self.u) {
                            if self.A[row][col] != Octet::zero() {
                                non_zero += 1;
                            }
                            if non_zero > r {
                                break;
                            }
                        }
                        if non_zero == r {
                            chosen_row = Some(row);
                            break;
                        }
                    }
                }
            }
            else {
                // TODO XXX !!!!!!: need to sort by something called "original degree"
                let mut chosen_hdpc = None;
                let mut chosen_non_hdpc = None;
                for row in self.i..self.L {
                    let mut non_zero = 0;
                    for col in self.i..(self.L - self.u) {
                        if self.A[row][col] != Octet::zero() {
                            non_zero += 1;
                        }
                        if non_zero > r {
                            break;
                        }
                    }
                    if non_zero == r {
                        if hdpc_rows[row] {
                            chosen_hdpc = Some(row);
                        }
                        else {
                            chosen_non_hdpc = Some(row);
                            break;
                        }
                    }
                }
                if chosen_non_hdpc != None {
                    chosen_row = chosen_non_hdpc;
                }
                else {
                    chosen_row = chosen_hdpc;
                }
            }

            // See paragraph beginning: "After the row is chosen in this step..."
            // Reorder rows
            let temp = self.i;
            let chosen_row = chosen_row.unwrap();
            self.swap_rows(temp, chosen_row);
            self.X.swap(temp, chosen_row);
            hdpc_rows.swap(temp, chosen_row);
            // Reorder columns
            let mut swapped_columns = 0;
            for col in self.i..(self.A[self.i].len() - self.u) {
                if self.A[self.i][col] != Octet::zero() {
                    let dest;
                    if swapped_columns == 0 {
                        dest = self.i;
                    }
                    else {
                        dest = self.L - self.u - swapped_columns;
                    }
                    self.swap_columns(dest, col);
                    // Also apply to X
                    for row in 0..self.X.len() {
                        self.X[row].swap(dest, col);
                    }
                    swapped_columns += 1;
                    if swapped_columns == r {
                        break;
                    }
                }
            }
            // Zero out leading value in following rows
            let temp = self.i;
            for row in (self.i + 1)..self.A.len() {
                if self.A[row][temp] != Octet::zero() {
                    // Addition is equivalent to subtraction
                    let beta = &self.A[row][temp] / &self.A[temp][temp];
                    self.fma_rows(temp, row, beta);
                }
            }

            self.i += 1;
            self.u += r - 1;
            // TODO: should only run this in debug mode
            self.first_phase_verify();
        }

        return true;
    }

    // See section 5.4.2.2. Verifies the two all-zeros submatrices and the identity submatrix
    fn first_phase_verify(&self) {
        for row in 0..self.i {
            for col in 0..self.i {
                if row == col {
                    assert_eq!(Octet::one(), self.A[row][col]);
                }
                else {
                    assert_eq!(Octet::zero(), self.A[row][col]);
                }
            }
        }
        assert!(self.all_zeroes(0, self.i, self.i, self.A.len() - self.u));
        assert!(self.all_zeroes(self.i, self.A.len(), 0, self.i));
    }

    // Second phase (section 5.4.2.3)
    #[allow(non_snake_case)]
    fn second_phase(&mut self) -> bool {
        // TODO: should only run this in debug mode
        self.second_phase_verify();

        let rows_to_discard = self.i..self.X.len();
        let cols_to_discard = self.i..self.X[0].len();
        self.X.drain(rows_to_discard);
        for row in 0..self.X.len() {
            self.X[row].drain(cols_to_discard.clone());
        }

        // Convert U_lower to row echelon form
        let temp = self.i;
        let size = self.u;
        if !self.reduce_to_row_echelon(temp, temp, size) {
            return false;
        }

        // Perform backwards elimination
        self.backwards_elimination(temp, temp, size);

        return true;
    }

    // Verifies that X is lower triangular. See section 5.4.2.3
    fn second_phase_verify(&self) {
        for row in 0..self.i {
            for col in (row + 1)..self.i {
                assert_eq!(Octet::zero(), self.X[row][col]);
            }
        }
    }

    // Third phase (section 5.4.2.4)
    #[allow(non_snake_case)]
    fn third_phase(&mut self) {
        // TODO: should only run this in debug mode
        self.third_phase_verify();
    }

    fn third_phase_verify(&self) {
        for row in 0..self.A.len() {
            for col in 0..self.A[row].len() {
                if row < self.i && col >= self.A[row].len() - self.u {
                    // element is in U_upper, which can have arbitrary values at this point
                    continue;
                }
                // The rest of A should be identity matrix
                if row == col {
                    assert_eq!(Octet::one(), self.A[row][col]);
                }
                else {
                    assert_eq!(Octet::zero(), self.A[row][col]);
                }
            }
        }
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
            for l in 1..j {
                let temp = self.A[j][l].clone();
                if temp != Octet::zero() {
                    self.fma_rows(l, j, temp);
                }
            }
        }
    }

    // Reduces the size x size submatrix, starting at row_offset and col_offset as the upper left
    // corner, to row echelon form
    fn reduce_to_row_echelon(&mut self, row_offset: usize, col_offset: usize, size: usize) -> bool {
        for i in 0..size {
            // Swap a row with leading coefficient i into place
            for j in (row_offset + i)..self.A.len() {
                if self.A[j][col_offset + i] != Octet::zero() {
                    self.swap_rows(row_offset + i, j);
                    break;
                }
            }

            if self.A[row_offset + i][col_offset + i] == Octet::zero() {
                // If all following rows are zero in this column, then matrix is singular
                return false;
            }

            // Scale leading coefficient to 1
            if self.A[row_offset + i][col_offset + i] != Octet::one() {
                let element_inverse = Octet::one() / self.A[row_offset + i][col_offset + i].clone();
                self.mul_row(row_offset + i, element_inverse);
            }

            // Zero out all following elements in i'th column
            for j in (row_offset + i + 1)..self.A.len() {
                if self.A[j][col_offset + i] != Octet::zero() {
                    let scalar = self.A[j][col_offset + i].clone();
                    self.fma_rows(row_offset + i, j, scalar);
                }
            }
        }

        return true;
    }

    // Performs backwards elimination in a size x size submatrix, starting at
    // row_offset and col_offset as the upper left corner of the submatrix
    fn backwards_elimination(&mut self, row_offset: usize, col_offset: usize, size: usize) {
        // Perform backwards elimination
        for i in (0..size).rev() {
            // Zero out all preceding elements in i'th column
            for j in 0..i {
                if self.A[row_offset + j][col_offset + i] != Octet::zero() {
                    let scalar = self.A[row_offset + j][col_offset + i].clone();
                    self.fma_rows(row_offset + i, row_offset + j, scalar);
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

        if !self.second_phase() {
            return None;
        }

        self.third_phase();
        self.fourth_phase();
        self.fifth_phase();

        // TODO: remove this part. It's only here because some phases aren't implemented yet
        let size = self.A.len();
        if !self.reduce_to_row_echelon(0, 0, size) {
            return None;
        }
        // Perform backwards elimination
        self.backwards_elimination(0, 0, size);
        // TODO: end of todo

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
