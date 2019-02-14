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
use octet::Octet;
use octets::count_ones_and_nonzeros;
use arraymap::ArrayMap;
use util::get_both_indices;
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

struct FirstPhaseRowSelectionStats {
    non_zeros_per_row: ArrayMap<usize>,
    ones_per_row: ArrayMap<usize>,
    start_col: usize,
    end_col: usize
}

impl FirstPhaseRowSelectionStats {
    #[inline(never)]
    pub fn new(matrix: &OctetMatrix) -> FirstPhaseRowSelectionStats {
        let mut result = FirstPhaseRowSelectionStats {
            non_zeros_per_row: ArrayMap::new(0, matrix.height()),
            ones_per_row: ArrayMap::new(0, matrix.height()),
            start_col: 0,
            end_col: matrix.width()
        };

        for i in 0..matrix.height() {
            result.recompute_row(i, matrix);
        }

        result
    }

    pub fn swap_rows(&mut self, i: usize, j: usize) {
        self.non_zeros_per_row.swap(i, j);
        self.ones_per_row.swap(i, j);
    }

    // Recompute all stored statistics for the given row
    pub fn recompute_row(&mut self, row: usize, matrix: &OctetMatrix) {
        let (ones, non_zero) = count_ones_and_nonzeros(&matrix.get_row(row)[self.start_col..self.end_col]);
        self.non_zeros_per_row.insert(row, non_zero);
        self.ones_per_row.insert(row, ones);
    }

    // Set the valid columns, and recalculate statistics
    #[inline(never)]
    pub fn resize(&mut self, start_row: usize, end_row: usize, start_col: usize, end_col: usize, matrix: &OctetMatrix) {
        // Only shrinking is supported
        assert!(start_col > self.start_col);
        assert!(end_col <= self.end_col);

        for row in start_row..end_row {
            for col in self.start_col..start_col {
                if matrix.get(row, col) == Octet::one() {
                    let old = self.ones_per_row.get(row);
                    self.ones_per_row.insert(row, old - 1);
                }
                if matrix.get(row, col) != Octet::zero() {
                    let old = self.non_zeros_per_row.get(row);
                    self.non_zeros_per_row.insert(row, old - 1);
                }
            }

            for col in end_col..self.end_col {
                if matrix.get(row, col) == Octet::one() {
                    let old = self.ones_per_row.get(row);
                    self.ones_per_row.insert(row, old - 1);
                }
                if matrix.get(row, col) != Octet::zero() {
                    let old = self.non_zeros_per_row.get(row);
                    self.non_zeros_per_row.insert(row, old - 1);
                }
            }
        }

        self.start_col = start_col;
        self.end_col = end_col;
    }

    // Helper method for decoder phase 1
    // selects from [start_row, end_row) reading [start_col, end_col)
    // Returns (rows with two 1s, a row with two values > 1,
    // mapping from row number to number of non-zero values, "r" minimum positive number of non-zero values a row has)
    pub fn first_phase_selection(&self, start_row: usize, end_row: usize) -> (Vec<usize>, Option<usize>, ArrayMap<usize>, Option<usize>) {
        let mut rows_with_two_ones = vec![];
        let mut row_with_two_greater_than_one = None;
        let mut r = std::usize::MAX;
        for row in start_row..end_row {
            let non_zero = self.non_zeros_per_row.get(row);
            let ones = self.ones_per_row.get(row);
            if non_zero > 0 && non_zero < r {
                r = non_zero;
            }
            if non_zero == 2 && ones != 2 {
                row_with_two_greater_than_one = Some(row);
            }
            if ones == 2 {
                rows_with_two_ones.push(row);
            }
        }

        if r < std::usize::MAX {
            (rows_with_two_ones, row_with_two_greater_than_one, self.non_zeros_per_row.clone(), Some(r))
        }
        else {
            (rows_with_two_ones, row_with_two_greater_than_one, self.non_zeros_per_row.clone(), None)
        }
    }
}

// See section 5.4.2.1
#[allow(non_snake_case)]
pub struct IntermediateSymbolDecoder {
    A: OctetMatrix,
    X: OctetMatrix,
    D: Vec<Symbol>,
    c: Vec<usize>,
    d: Vec<usize>,
    i: usize,
    u: usize,
    L: usize,
    num_source_symbols: u32,
    debug_symbol_mul_ops: u32,
    debug_symbol_add_ops: u32,
    debug_symbol_mul_ops_by_phase: Vec<u32>,
    debug_symbol_add_ops_by_phase: Vec<u32>
}

impl IntermediateSymbolDecoder {
    pub fn new(matrix: &OctetMatrix, symbols: &Vec<Symbol>, num_source_symbols: u32) -> IntermediateSymbolDecoder {
        // TODO: implement for non-square matrices
        assert_eq!(matrix.width(), symbols.len());
        assert_eq!(matrix.height(), symbols.len());
        let mut c = Vec::with_capacity(matrix.width());
        let mut d = Vec::with_capacity(matrix.width());
        for i in 0..matrix.width() {
            c.push(i);
            d.push(i);
        }

        IntermediateSymbolDecoder {
            A: matrix.clone(),
            X: matrix.clone(),
            D: symbols.clone(),
            c,
            d,
            i: 0,
            u: num_pi_symbols(num_source_symbols) as usize,
            L: num_intermediate_symbols(num_source_symbols) as usize,
            num_source_symbols,
            debug_symbol_mul_ops: 0,
            debug_symbol_add_ops: 0,
            debug_symbol_mul_ops_by_phase: vec![0; 5],
            debug_symbol_add_ops_by_phase: vec![0; 5]
        }
    }

    // Returns true iff all elements in A between [start_row, end_row)
    // and [start_column, end_column) are zero
    #[cfg(debug_assertions)]
    fn all_zeroes(&self, start_row: usize, end_row: usize, start_column: usize, end_column: usize) -> bool {
        for row in start_row..end_row {
            for column in start_column..end_column {
                if self.A.get(row, column) != Octet::zero() {
                    return false;
                }
            }
        }
        return true;
    }

    #[inline(never)]
    fn first_phase_graph_substep(&self, rows_with_two_ones: &Vec<usize>, hdpc_rows: &Vec<bool>) -> usize {
        let mut g = Graph::new_undirected();
        let mut node_lookup = ArrayMap::new(self.i, self.L - self.u);
        for col in self.i..(self.L - self.u) {
            let node = g.add_node(col);
            node_lookup.insert(col, node);
        }

        for row in rows_with_two_ones.iter() {
            if hdpc_rows[*row] {
                continue;
            }
            let mut ones = [0; 2];
            let mut found = 0;
            for col in self.i..(self.L - self.u) {
                // "The following graph defined by the structure of V is used in determining which
                // row of A is chosen. The columns that intersect V are the nodes in the graph,
                // and the rows that have exactly 2 nonzero entries in V and are not HDPC rows
                // are the edges of the graph that connect the two columns (nodes) in the positions
                // of the two ones."
                // This part of the matrix is over GF(2), so "nonzero entries" is equivalent to "ones"
                if self.A.get(*row, col) == Octet::one() {
                    ones[found] = col;
                    found += 1;
                }
                if found == 2 {
                    break;
                }
            }
            let node1 = node_lookup.get(ones[0]);
            let node2 = node_lookup.get(ones[1]);
            g.add_edge(node1, node2, *row);
        }

        let connected_components = condensation(g.clone(), true);
        let mut row_to_component_size = ArrayMap::new(self.i, self.L);
        for index in connected_components.node_indices() {
            let cols = connected_components.node_weight(index).unwrap();
            for col in cols {
                for edge in g.edges(node_lookup.get(*col)) {
                    row_to_component_size.insert(*edge.weight(), cols.len());
                }
            }
        }

        let mut chosen_component_size = 0;
        let mut chosen_row= self.L + 1;
        for row in rows_with_two_ones {
            let row = *row;
            if hdpc_rows[row] {
                continue;
            }
            if row_to_component_size.get(row) > chosen_component_size {
                chosen_row = row;
                chosen_component_size = row_to_component_size.get(row);
            }
        }
        assert_ne!(chosen_row, self.L + 1);
        chosen_row
    }

    // Performs the column swapping substep of first phase, after the row has been chosen
    #[inline(never)]
    fn first_phase_swap_columns_substep(&mut self, r: usize) {
        let mut swapped_columns = 0;
        for col in self.i..(self.A.width() - self.u) {
            if self.A.get(self.i, col) != Octet::zero() {
                let dest;
                if swapped_columns == 0 {
                    dest = self.i;
                }
                else {
                    dest = self.L - self.u - swapped_columns;
                }
                self.swap_columns(dest, col);
                // Also apply to X
                self.X.swap_columns(dest, col);
                swapped_columns += 1;
                if swapped_columns == r {
                    break;
                }
            }
        }
    }

    #[inline(never)]
    fn first_phase_original_degree_substep(&self, original_degree: &ArrayMap<usize>, non_zero_counts: &ArrayMap<usize>, hdpc_rows: &Vec<bool>, r: usize) -> usize {
        let mut chosen_hdpc = None;
        let mut chosen_hdpc_original_degree = self.L + 1;
        let mut chosen_non_hdpc = None;
        let mut chosen_non_hdpc_original_degree = self.L + 1;
        for row in self.i..self.L {
            let non_zero = non_zero_counts.get(row);
            let row_original_degree = original_degree.get(row);
            if non_zero == r {
                if hdpc_rows[row] {
                    if row_original_degree < chosen_hdpc_original_degree {
                        chosen_hdpc = Some(row);
                        chosen_hdpc_original_degree = row_original_degree;
                    }
                }
                else if row_original_degree < chosen_non_hdpc_original_degree {
                    chosen_non_hdpc = Some(row);
                    chosen_non_hdpc_original_degree = row_original_degree;
                }
            }
        }
        if chosen_non_hdpc != None {
            return chosen_non_hdpc.unwrap();
        }
        else {
            return chosen_hdpc.unwrap();
        }
    }

    // First phase (section 5.4.2.2)
    #[allow(non_snake_case)]
    #[inline(never)]
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

        let mut hdpc_rows = vec![false; self.A.height()];
        for row in S..(S + H) {
            hdpc_rows[row as usize] = true;
        }

        let mut selection_helper = FirstPhaseRowSelectionStats::new(&self.A);

        // Original degree is the degree of each row before processing begins
        let (_, _, original_degree, _) = selection_helper.first_phase_selection(0, self.L);


        while self.i + self.u < self.L {
            // Calculate r
            // "Let r be the minimum integer such that at least one row of A has
            // exactly r nonzeros in V."
            let (rows_with_two_ones, row_with_two_greater_than_one, non_zero_counts, r) =
                selection_helper.first_phase_selection(self.i, self.L);

            if r == None {
                return false;
            }
            let r = r.unwrap();

            let chosen_row;
            if r == 2 {
                // See paragraph starting "If r = 2 and there is a row with exactly 2 ones in V..."
                if rows_with_two_ones.len() > 0 {
                    #[cfg(debug_assertions)]
                    self.first_phase_graph_substep_verify(self.i, self.L, &hdpc_rows, &rows_with_two_ones, &non_zero_counts);
                    chosen_row = Some(self.first_phase_graph_substep(&rows_with_two_ones, &hdpc_rows));
                }
                else {
                    // See paragraph starting "If r = 2 and there is no row with exactly 2 ones in V"
                    chosen_row = row_with_two_greater_than_one;
                }
            }
            else {
                chosen_row = Some(self.first_phase_original_degree_substep(&original_degree, &non_zero_counts, &hdpc_rows, r));
            }

            // See paragraph beginning: "After the row is chosen in this step..."
            // Reorder rows
            let temp = self.i;
            let chosen_row = chosen_row.unwrap();
            self.swap_rows(temp, chosen_row);
            self.X.swap_rows(temp, chosen_row);
            selection_helper.swap_rows(temp, chosen_row);
            hdpc_rows.swap(temp, chosen_row);
            // Reorder columns
            self.first_phase_swap_columns_substep(r);
            // Zero out leading value in following rows
            let temp = self.i;
            for row in (self.i + 1)..self.A.height() {
                if self.A.get(row, temp) != Octet::zero() {
                    // Addition is equivalent to subtraction
                    let beta = &self.A.get(row, temp) / &self.A.get(temp, temp);
                    self.fma_rows(temp, row, beta);
                    selection_helper.recompute_row(row, &self.A);
                }
            }

            self.i += 1;
            self.u += r - 1;
            selection_helper.resize(self.i, self.L, self.i, self.L - self.u, &self.A);
            #[cfg(debug_assertions)]
            self.first_phase_verify();
        }

        self.record_symbol_ops(0);
        return true;
    }

    // Verify there there are no non-HPDC rows with exactly two non-zero entries, greater than one
    #[inline(never)]
    #[cfg(debug_assertions)]
    fn first_phase_graph_substep_verify(&self, start_row: usize, end_row: usize, hdpc_rows: &Vec<bool>, rows_with_two_ones: &Vec<usize>, non_zeros: &ArrayMap<usize>) {
        for row in start_row..end_row {
            if non_zeros.get(row) == 2 {
                assert!(rows_with_two_ones.contains(&row) || hdpc_rows[row]);
            }
        }
    }

    // See section 5.4.2.2. Verifies the two all-zeros submatrices and the identity submatrix
    #[inline(never)]
    #[cfg(debug_assertions)]
    fn first_phase_verify(&self) {
        for row in 0..self.i {
            for col in 0..self.i {
                if row == col {
                    assert_eq!(Octet::one(), self.A.get(row, col));
                }
                else {
                    assert_eq!(Octet::zero(), self.A.get(row, col));
                }
            }
        }
        assert!(self.all_zeroes(0, self.i, self.i, self.A.height() - self.u));
        assert!(self.all_zeroes(self.i, self.A.height(), 0, self.i));
    }

    // Second phase (section 5.4.2.3)
    #[allow(non_snake_case)]
    #[inline(never)]
    fn second_phase(&mut self) -> bool {
        #[cfg(debug_assertions)]
        self.second_phase_verify();

        self.X.resize(self.i, self.i);

        // Convert U_lower to row echelon form
        let temp = self.i;
        let size = self.u;
        if !self.reduce_to_row_echelon(temp, temp, size) {
            return false;
        }

        // Perform backwards elimination
        self.backwards_elimination(temp, temp, size);

        self.record_symbol_ops(1);
        return true;
    }

    // Verifies that X is lower triangular. See section 5.4.2.3
    #[inline(never)]
    #[cfg(debug_assertions)]
    fn second_phase_verify(&self) {
        for row in 0..self.i {
            for col in (row + 1)..self.i {
                assert_eq!(Octet::zero(), self.X.get(row, col));
            }
        }
    }

    // Third phase (section 5.4.2.4)
    #[allow(non_snake_case)]
    #[inline(never)]
    fn third_phase(&mut self) {
        #[cfg(debug_assertions)]
        self.third_phase_verify();

        // A[0..i][..] = X * A[0..i][..]
        self.A.mul_assign_submatrix(&self.X, self.i);

        // Now apply the same operations to D.
        // Note that X is lower triangular, so the row must be processed last to first
        for row in (0..self.i).rev() {
            if self.X.get(row, row) != Octet::one() {
                self.debug_symbol_mul_ops += 1;
                self.D[self.d[row]].mulassign_scalar(&self.X.get(row, row));
            }

            for col in 0..row {
                if self.X.get(row, col) == Octet::zero() {
                    continue;
                }
                if self.X.get(row, col) == Octet::one() {
                    self.debug_symbol_add_ops += 1;
                    let (dest, temp) = get_both_indices(&mut self.D, self.d[row], self.d[col]);
                    *dest += temp;
                }
                else {
                    self.debug_symbol_mul_ops += 1;
                    self.debug_symbol_add_ops += 1;
                    let (dest, temp) = get_both_indices(&mut self.D, self.d[row], self.d[col]);
                    dest.fused_addassign_mul_scalar(temp, &self.X.get(row, col));
                }
            }
        }

        self.record_symbol_ops(2);

        #[cfg(debug_assertions)]
        self.third_phase_verify_end();
    }

    #[inline(never)]
    #[cfg(debug_assertions)]
    fn third_phase_verify(&self) {
        for row in 0..self.A.height() {
            for col in 0..self.A.width() {
                if row < self.i && col >= self.A.width() - self.u {
                    // element is in U_upper, which can have arbitrary values at this point
                    continue;
                }
                // The rest of A should be identity matrix
                if row == col {
                    assert_eq!(Octet::one(), self.A.get(row, col));
                }
                else {
                    assert_eq!(Octet::zero(), self.A.get(row, col));
                }
            }
        }
    }

    #[inline(never)]
    #[cfg(debug_assertions)]
    fn third_phase_verify_end(&self) {
        for row in 0..self.i {
            for col in 0..self.i {
                assert_eq!(self.X.get(row, col), self.A.get(row, col));
            }
        }
    }

    // Fourth phase (section 5.4.2.5)
    #[allow(non_snake_case)]
    #[inline(never)]
    fn fourth_phase(&mut self) {
        for i in 0..self.i {
            for j in 0..self.u {
                let b = self.A.get(i, j + self.i);
                if b != Octet::zero() {
                    let temp = self.i;
                    self.fma_rows(temp + j, i, b);
                }
            }
        }

        self.record_symbol_ops(3);

        #[cfg(debug_assertions)]
        self.fourth_phase_verify();
    }

    #[inline(never)]
    #[cfg(debug_assertions)]
    fn fourth_phase_verify(&self) {
        //    ---------> i u <------
        //  | +-----------+--------+
        //  | |\          |        |
        //  | |  \ Zeros  | Zeros  |
        //  v |     \     |        |
        //  i |  X     \  |        |
        //  u +---------- +--------+
        //  ^ |           |        |
        //  | | All Zeros |   I    |
        //  | |           |        |
        //    +-----------+--------+
        // Same assertion about X being equal to the upper left of A
        #[cfg(debug_assertions)]
        self.third_phase_verify_end();
        assert!(self.all_zeroes(0, self.i, self.L - self.u, self.L));
        assert!(self.all_zeroes(self.L - self.u, self.L, 0, self.i));
        for row in (self.L - self.u)..self.L {
            for col in (self.L - self.u)..self.L {
                if row == col {
                    assert_eq!(Octet::one(), self.A.get(row, col));
                }
                else {
                    assert_eq!(Octet::zero(), self.A.get(row, col));
                }
            }
        }
    }

    // Fifth phase (section 5.4.2.6)
    #[allow(non_snake_case)]
    #[inline(never)]
    fn fifth_phase(&mut self) {
        // "For j from 1 to i". Note that A is 1-indexed in the spec, and ranges are inclusive,
        // this is means [1, i], which is equal to [0, i)
        for j in 0..self.i as usize {
            if self.A.get(j, j) != Octet::one() {
                let temp = self.A.get(j, j);
                self.mul_row(j, Octet::one() / temp)
            }
            // "For l from 1 to j-1". This means the lower triangular columns, not including the
            // diagonal, which is [0, j)
            for l in 0..j {
                let temp = self.A.get(j, l);
                if temp != Octet::zero() {
                    self.fma_rows(l, j, temp);
                }
            }
        }

        self.record_symbol_ops(4);

        #[cfg(debug_assertions)]
        self.fifth_phase_verify();
    }

    #[inline(never)]
    #[cfg(debug_assertions)]
    fn fifth_phase_verify(&self) {
        assert_eq!(self.L, self.A.height());
        for row in 0..self.L {
            assert_eq!(self.L, self.A.width());
            for col in 0..self.L {
                if row == col {
                    assert_eq!(Octet::one(), self.A.get(row, col));
                }
                else {
                    assert_eq!(Octet::zero(), self.A.get(row, col));
                }
            }
        }
    }

    fn record_symbol_ops(&mut self, phase: usize) {
        self.debug_symbol_add_ops_by_phase[phase] = self.debug_symbol_add_ops;
        self.debug_symbol_mul_ops_by_phase[phase] = self.debug_symbol_mul_ops;
        for i in 0..phase {
            self.debug_symbol_add_ops_by_phase[phase] -= self.debug_symbol_add_ops_by_phase[i];
            self.debug_symbol_mul_ops_by_phase[phase] -= self.debug_symbol_mul_ops_by_phase[i];
        }
    }

    // Reduces the size x size submatrix, starting at row_offset and col_offset as the upper left
    // corner, to row echelon form
    #[inline(never)]
    fn reduce_to_row_echelon(&mut self, row_offset: usize, col_offset: usize, size: usize) -> bool {
        for i in 0..size {
            // Swap a row with leading coefficient i into place
            for j in (row_offset + i)..self.A.height() {
                if self.A.get(j, col_offset + i) != Octet::zero() {
                    self.swap_rows(row_offset + i, j);
                    break;
                }
            }

            if self.A.get(row_offset + i, col_offset + i) == Octet::zero() {
                // If all following rows are zero in this column, then matrix is singular
                return false;
            }

            // Scale leading coefficient to 1
            if self.A.get(row_offset + i, col_offset + i) != Octet::one() {
                let element_inverse = Octet::one() / self.A.get(row_offset + i, col_offset + i);
                self.mul_row(row_offset + i, element_inverse);
            }

            // Zero out all following elements in i'th column
            for j in (row_offset + i + 1)..self.A.height() {
                if self.A.get(j, col_offset + i) != Octet::zero() {
                    let scalar = self.A.get(j, col_offset + i);
                    self.fma_rows(row_offset + i, j, scalar);
                }
            }
        }

        return true;
    }

    // Performs backwards elimination in a size x size submatrix, starting at
    // row_offset and col_offset as the upper left corner of the submatrix
    #[inline(never)]
    fn backwards_elimination(&mut self, row_offset: usize, col_offset: usize, size: usize) {
        // Perform backwards elimination
        for i in (0..size).rev() {
            // Zero out all preceding elements in i'th column
            for j in 0..i {
                if self.A.get(row_offset + j, col_offset + i) != Octet::zero() {
                    let scalar = self.A.get(row_offset + j, col_offset + i);
                    self.fma_rows(row_offset + i, row_offset + j, scalar);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_symbol_mul_ops(&self) -> u32 {
        self.debug_symbol_mul_ops
    }

    #[allow(dead_code)]
    pub fn get_symbol_add_ops(&self) -> u32 {
        self.debug_symbol_add_ops
    }

    #[allow(dead_code)]
    pub fn get_symbol_mul_ops_by_phase(&self) -> Vec<u32> {
        self.debug_symbol_mul_ops_by_phase.clone()
    }

    #[allow(dead_code)]
    pub fn get_symbol_add_ops_by_phase(&self) -> Vec<u32> {
        self.debug_symbol_add_ops_by_phase.clone()
    }

    // Helper operations to apply operations to A, also to D
    fn mul_row(&mut self, i: usize, beta: Octet) {
        self.debug_symbol_mul_ops += 1;
        self.D[self.d[i]].mulassign_scalar(&beta);
        for j in 0..self.L {
            let temp = &self.A.get(i, j) * &beta;
            self.A.set(i, j, temp);
        }
    }

    fn fma_rows(&mut self, i: usize, iprime: usize, beta: Octet) {
        if beta == Octet::one() {
            self.debug_symbol_add_ops += 1;
            let (dest, temp) = get_both_indices(&mut self.D, self.d[iprime], self.d[i]);
            *dest += temp;
        }
        else {
            self.debug_symbol_add_ops += 1;
            self.debug_symbol_mul_ops += 1;
            let (dest, temp) = get_both_indices(&mut self.D, self.d[iprime], self.d[i]);
            dest.fused_addassign_mul_scalar(&temp, &beta);
        }
        self.A.fma_rows(iprime, i, &beta);
    }

    fn swap_rows(&mut self, i: usize, iprime: usize) {
        self.A.swap_rows(i, iprime);
        self.d.swap(i, iprime);
    }

    fn swap_columns(&mut self, j: usize, jprime: usize) {
        self.A.swap_columns(j, jprime);
        self.c.swap(j, jprime);
    }

    #[inline(never)]
    pub fn execute(&mut self) -> Option<Vec<Symbol>> {
        if !self.first_phase() {
            return None
        }

        if !self.second_phase() {
            return None;
        }

        self.third_phase();
        self.fourth_phase();
        self.fifth_phase();

        // See end of section 5.4.2.1
        let mut index_mapping = ArrayMap::new(0, self.L);
        for i in 0..self.L {
            index_mapping.insert(self.c[i], self.d[i]);
        }
        let mut result = Vec::with_capacity(self.L);
        for i in 0..self.L {
            result.push(self.D[index_mapping.get(i)].clone());
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
    use symbol::Symbol;
    use systematic_constants::extended_source_block_symbols;
    use constraint_matrix::generate_constraint_matrix;
    use octet::Octet;
    use base::IntermediateSymbolDecoder;

    #[test]
    fn operations_per_symbol() {
        Octet::static_init();
        for elements in [10, 100].iter() {
            let num_symbols = extended_source_block_symbols(*elements);
            let a = generate_constraint_matrix(num_symbols, 0..num_symbols);
            let symbols = vec![Symbol::zero(1); a.width()];
            let mut decoder = IntermediateSymbolDecoder::new(&a, &symbols, num_symbols);
            decoder.execute();
            assert!((decoder.get_symbol_mul_ops() as f64 / num_symbols as f64) < 30.0);
            assert!((decoder.get_symbol_add_ops() as f64 / num_symbols as f64) < 50.0);
        }
    }
}
