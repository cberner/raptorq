use crate::arraymap::{ArrayMap, BoolArrayMap};
use crate::arraymap::UsizeArrayMap;
use crate::matrix::OctetMatrix;
use crate::octet::Octet;
use crate::octets::count_ones_and_nonzeros;
use crate::octets::mulassign_scalar;
use crate::symbol::Symbol;
use crate::systematic_constants::num_hdpc_symbols;
use crate::systematic_constants::num_intermediate_symbols;
use crate::systematic_constants::num_ldpc_symbols;
use crate::systematic_constants::num_pi_symbols;
use crate::util::get_both_indices;

struct FirstPhaseRowSelectionStats {
    original_degree: UsizeArrayMap,
    non_zeros_per_row: UsizeArrayMap,
    ones_per_row: UsizeArrayMap,
    non_zeros_histogram: UsizeArrayMap,
    hdpc_rows: Vec<bool>,
    start_col: usize,
    end_col: usize,
    start_row: usize
}

impl FirstPhaseRowSelectionStats {
    #[inline(never)]
    #[allow(non_snake_case)]
    pub fn new(matrix: &OctetMatrix, end_col: usize, num_source_symbols: u32) -> FirstPhaseRowSelectionStats {
        let S = num_ldpc_symbols(num_source_symbols);
        let H = num_hdpc_symbols(num_source_symbols);

        // See section 5.3.3.4.2, Figure 5.
        let mut hdpc_rows = vec![false; matrix.height()];
        for row in S..(S + H) {
            hdpc_rows[row as usize] = true;
        }

        let mut result = FirstPhaseRowSelectionStats {
            original_degree: UsizeArrayMap::new(0, 0),
            non_zeros_per_row: UsizeArrayMap::new(0, matrix.height()),
            ones_per_row: UsizeArrayMap::new(0, matrix.height()),
            non_zeros_histogram: UsizeArrayMap::new(0, end_col + 1),
            hdpc_rows,
            start_col: 0,
            end_col,
            start_row: 0
        };

        for row in 0..matrix.height() {
            let (ones, non_zero) = count_ones_and_nonzeros(&matrix.get_row(row)[0..end_col]);
            result.non_zeros_per_row.insert(row, non_zero);
            result.ones_per_row.insert(row, ones);
            result.non_zeros_histogram.increment(non_zero);
        }
        // Original degree is the degree of each row before processing begins
        result.original_degree = result.non_zeros_per_row.clone();

        result
    }

    pub fn swap_rows(&mut self, i: usize, j: usize) {
        self.non_zeros_per_row.swap(i, j);
        self.ones_per_row.swap(i, j);
        self.original_degree.swap(i, j);
        self.hdpc_rows.swap(i, j);
    }

    // Recompute all stored statistics for the given row
    pub fn recompute_row(&mut self, row: usize, matrix: &OctetMatrix) {
        let (ones, non_zero) = count_ones_and_nonzeros(&matrix.get_row(row)[self.start_col..self.end_col]);
        self.non_zeros_histogram.decrement(self.non_zeros_per_row.get(row));
        self.non_zeros_histogram.increment(non_zero);
        self.non_zeros_per_row.insert(row, non_zero);
        self.ones_per_row.insert(row, ones);
    }

    pub fn eliminate_leading_value(&mut self, row: usize, value: &Octet) {
        debug_assert_ne!(*value, Octet::zero());
        if *value == Octet::one() {
            self.ones_per_row.decrement(row);
        }
        let non_zeros = self.non_zeros_per_row.get(row);
        self.non_zeros_histogram.decrement(non_zeros);
        self.non_zeros_histogram.increment(non_zeros - 1);
        self.non_zeros_per_row.decrement(row);
    }

    // Set the valid columns, and recalculate statistics
    #[inline(never)]
    pub fn resize(&mut self, start_row: usize, end_row: usize, start_col: usize, end_col: usize, matrix: &OctetMatrix) {
        // Only shrinking is supported
        assert!(start_col > self.start_col);
        assert!(end_col <= self.end_col);
        assert_eq!(self.start_row, start_row - 1);

        self.non_zeros_histogram.decrement(self.non_zeros_per_row.get(self.start_row));

        for row in start_row..end_row {
            for col in self.start_col..start_col {
                if matrix.get(row, col) == Octet::one() {
                    self.ones_per_row.decrement(row);
                }
                if matrix.get(row, col) != Octet::zero() {
                    let non_zeros = self.non_zeros_per_row.get(row);
                    self.non_zeros_histogram.decrement(non_zeros);
                    self.non_zeros_histogram.increment(non_zeros - 1);
                    self.non_zeros_per_row.decrement(row);
                }
            }

            for col in end_col..self.end_col {
                if matrix.get(row, col) == Octet::one() {
                    self.ones_per_row.decrement(row);
                }
                if matrix.get(row, col) != Octet::zero() {
                    let non_zeros = self.non_zeros_per_row.get(row);
                    self.non_zeros_histogram.decrement(non_zeros);
                    self.non_zeros_histogram.increment(non_zeros - 1);
                    self.non_zeros_per_row.decrement(row);
                }
            }
        }

        self.start_col = start_col;
        self.end_col = end_col;
        self.start_row = start_row;
    }

    #[inline(never)]
    fn first_phase_graph_substep_build_adjacency(&self, rows_with_two_ones: &Vec<usize>, matrix: &OctetMatrix) -> ArrayMap<Vec<(usize, usize)>> {
        let mut adjacent_nodes = ArrayMap::new(self.start_col, self.end_col);

        for row in rows_with_two_ones.iter() {
            if self.hdpc_rows[*row] {
                continue;
            }
            let mut ones = [0; 2];
            let mut found = 0;
            for col in self.start_col..self.end_col {
                // "The following graph defined by the structure of V is used in determining which
                // row of A is chosen. The columns that intersect V are the nodes in the graph,
                // and the rows that have exactly 2 nonzero entries in V and are not HDPC rows
                // are the edges of the graph that connect the two columns (nodes) in the positions
                // of the two ones."
                // This part of the matrix is over GF(2), so "nonzero entries" is equivalent to "ones"
                if matrix.get(*row, col) == Octet::one() {
                    ones[found] = col;
                    found += 1;
                }
                if found == 2 {
                    break;
                }
            }
            assert_eq!(found, 2);
            let first = adjacent_nodes.get_mut(ones[0]);
            if first == None {
                let mut new_nodes = Vec::with_capacity(10);
                new_nodes.push((ones[1], *row));
                adjacent_nodes.insert(ones[0], new_nodes);
            }
            else {
                first.unwrap().push((ones[1], *row));
            }
            let second = adjacent_nodes.get_mut(ones[1]);
            if second == None {
                let mut new_nodes = Vec::with_capacity(10);
                new_nodes.push((ones[0], *row));
                adjacent_nodes.insert(ones[1], new_nodes);
            }
            else {
                second.unwrap().push((ones[0], *row));
            }
        }

        return adjacent_nodes;
    }

    #[inline(never)]
    fn first_phase_graph_substep(&self, start_row: usize, end_row: usize, rows_with_two_ones: &Vec<usize>, matrix: &OctetMatrix) -> usize {
        let adjacent_nodes = self.first_phase_graph_substep_build_adjacency(rows_with_two_ones, matrix);
        let mut visited = BoolArrayMap::new(start_row, end_row);

        let mut examplar_largest_component_row = None;
        let mut largest_component_size = 0;

        let mut node_queue = Vec::with_capacity(10);
        for key in adjacent_nodes.keys() {
            let mut component_size = 0;
            let mut examplar_row = None;
            // Pick arbitrary node (column) to start
            node_queue.clear();
            node_queue.push(key);
            while !node_queue.is_empty() {
                let node = node_queue.pop().unwrap();
                if visited.get(node) {
                    continue;
                }
                visited.insert(node, true);
                let next_nodes = adjacent_nodes.get(node).unwrap();
                component_size += 1;
                for &(next_node, row) in next_nodes.iter() {
                    node_queue.push(next_node);
                    examplar_row = Some(row);
                }
            }

            if component_size > largest_component_size {
                examplar_largest_component_row = examplar_row;
                largest_component_size = component_size;
            }
        }

        return examplar_largest_component_row.unwrap();
    }

    #[inline(never)]
    fn first_phase_original_degree_substep(&self, start_row: usize, end_row: usize, r: usize) -> usize {
        let mut chosen_hdpc = None;
        let mut chosen_hdpc_original_degree = std::usize::MAX;
        let mut chosen_non_hdpc = None;
        let mut chosen_non_hdpc_original_degree = std::usize::MAX;
        for row in start_row..end_row {
            let non_zero = self.non_zeros_per_row.get(row);
            let row_original_degree = self.original_degree.get(row);
            if non_zero == r {
                if self.hdpc_rows[row] {
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

    // Verify there there are no non-HPDC rows with exactly two non-zero entries, greater than one
    #[inline(never)]
    #[cfg(debug_assertions)]
    fn first_phase_graph_substep_verify(&self, start_row: usize, end_row: usize, rows_with_two_ones: &Vec<usize>) {
        for row in start_row..end_row {
            if self.non_zeros_per_row.get(row) == 2 {
                assert!(rows_with_two_ones.contains(&row) || self.hdpc_rows[row]);
            }
        }
    }

    // Helper method for decoder phase 1
    // selects from [start_row, end_row) reading [start_col, end_col)
    // Returns (the chosen row, and "r" number of non-zero values the row has)
    pub fn first_phase_selection(&self, start_row: usize, end_row: usize, matrix: &OctetMatrix) -> (Option<usize>, Option<usize>) {
        let mut r = None;
        for i in 1..(self.end_col - self.start_col + 1) {
            if self.non_zeros_histogram.get(i) > 0 {
                r = Some(i);
                break;
            }
        }

        if r == None {
            return (None, None);
        }

        if r.unwrap() == 2 {
            let mut rows_with_two_ones = vec![];
            let mut row_with_two_greater_than_one = None;
            for row in start_row..end_row {
                let non_zero = self.non_zeros_per_row.get(row);
                let ones = self.ones_per_row.get(row);
                if non_zero == 2 && ones != 2 {
                    row_with_two_greater_than_one = Some(row);
                }
                if non_zero == 2 && ones == 2 {
                    rows_with_two_ones.push(row);
                }
            }

            // See paragraph starting "If r = 2 and there is a row with exactly 2 ones in V..."
            if rows_with_two_ones.len() > 0 {
                #[cfg(debug_assertions)]
                self.first_phase_graph_substep_verify(start_row, end_row, &rows_with_two_ones);
                return (Some(self.first_phase_graph_substep(start_row, end_row, &rows_with_two_ones, matrix)), r);
            }
            else {
                // See paragraph starting "If r = 2 and there is no row with exactly 2 ones in V"
                return (row_with_two_greater_than_one, r);
            }
        }
        else {
            return (Some(self.first_phase_original_degree_substep(start_row, end_row, r.unwrap())), r);
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
    pub fn new(matrix: OctetMatrix, symbols: Vec<Symbol>, num_source_symbols: u32) -> IntermediateSymbolDecoder {
        assert!(matrix.width() <= symbols.len());
        assert_eq!(matrix.height(), symbols.len());
        let mut c = Vec::with_capacity(matrix.width());
        let mut d = Vec::with_capacity(symbols.len());
        for i in 0..matrix.width() {
            c.push(i);
        }
        for i in 0..symbols.len() {
            d.push(i);
        }

        IntermediateSymbolDecoder {
            A: matrix.clone(),
            X: matrix,
            D: symbols,
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
                    dest = self.A.width() - self.u - swapped_columns;
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

        let mut selection_helper = FirstPhaseRowSelectionStats::new(&self.A, self.A.width() - self.u, self.num_source_symbols);

        while self.i + self.u < self.L {
            // Calculate r
            // "Let r be the minimum integer such that at least one row of A has
            // exactly r nonzeros in V."
            let (chosen_row, r) = selection_helper.first_phase_selection(self.i, self.A.height(), &self.A);

            if r == None {
                return false;
            }
            let r = r.unwrap();
            let chosen_row = chosen_row.unwrap();

            // See paragraph beginning: "After the row is chosen in this step..."
            // Reorder rows
            let temp = self.i;
            self.swap_rows(temp, chosen_row);
            self.X.swap_rows(temp, chosen_row);
            selection_helper.swap_rows(temp, chosen_row);
            // Reorder columns
            self.first_phase_swap_columns_substep(r);
            // Zero out leading value in following rows
            let temp = self.i;
            for row in (self.i + 1)..self.A.height() {
                let leading_value = self.A.get(row, temp);
                if leading_value != Octet::zero() {
                    // Addition is equivalent to subtraction
                    let beta = &leading_value / &self.A.get(temp, temp);
                    self.fma_rows(temp, row, beta);
                    if r == 1 {
                        // Hot path for r == 1, since it's very common due to maximum connected
                        // component selection, and recompute_row() is expensive
                        selection_helper.eliminate_leading_value(row, &leading_value);
                    }
                    else {
                        selection_helper.recompute_row(row, &self.A);
                    }
                }
            }

            self.i += 1;
            self.u += r - 1;
            selection_helper.resize(self.i, self.A.height(), self.i, self.A.width() - self.u, &self.A);
            #[cfg(debug_assertions)]
            self.first_phase_verify();
        }

        self.record_symbol_ops(0);
        return true;
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
        assert!(self.all_zeroes(0, self.i, self.i, self.A.width() - self.u));
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

        self.A.resize(self.L, self.L);

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
        assert!(self.all_zeroes(0, self.i, self.A.width() - self.u, self.A.width()));
        assert!(self.all_zeroes(self.A.height() - self.u, self.A.height(), 0, self.i));
        for row in (self.A.height() - self.u)..self.A.height() {
            for col in (self.A.width() - self.u)..self.A.width() {
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
        for row in 0..self.A.height() {
            assert_eq!(self.L, self.A.width());
            for col in 0..self.A.width() {
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
        mulassign_scalar(self.A.get_row_mut(i), &beta);
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
        let mut index_mapping = UsizeArrayMap::new(0, self.L);
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
pub fn fused_inverse_mul_symbols(matrix: OctetMatrix, symbols: Vec<Symbol>, num_source_symbols: u32) -> Option<Vec<Symbol>> {
    IntermediateSymbolDecoder::new(matrix, symbols, num_source_symbols).execute()
}

#[cfg(test)]
mod tests {
    use super::IntermediateSymbolDecoder;
    use crate::constraint_matrix::generate_constraint_matrix;
    use crate::symbol::Symbol;
    use crate::systematic_constants::extended_source_block_symbols;

    #[test]
    fn operations_per_symbol() {
        for &(elements, expected_mul_ops, expected_add_ops) in [(10, 35.0, 50.0), (100, 16.0, 35.0)].iter() {
            let num_symbols = extended_source_block_symbols(elements);
            let indices: Vec<u32> = (0..num_symbols).collect();
            let a = generate_constraint_matrix(num_symbols, &indices);
            let symbols = vec![Symbol::zero(1); a.width()];
            let mut decoder = IntermediateSymbolDecoder::new(a, symbols, num_symbols);
            decoder.execute();
            assert!((decoder.get_symbol_mul_ops() as f64 / num_symbols as f64) < expected_mul_ops, "mul ops per symbol = {}", (decoder.get_symbol_mul_ops() as f64 / num_symbols as f64));
            assert!((decoder.get_symbol_add_ops() as f64 / num_symbols as f64) < expected_add_ops, "add ops per symbol = {}", (decoder.get_symbol_add_ops() as f64 / num_symbols as f64));
        }
    }
}
