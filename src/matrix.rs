use std::ops::Mul;
use octet::Octet;
use octets::fused_addassign_mul_scalar;
use octets::add_assign;
use symbol::Symbol;

#[derive(Clone, Debug, PartialEq)]
pub struct OctetMatrix {
    height: usize,
    width: usize,
    elements: Vec<Vec<u8>>
}

impl OctetMatrix {
    pub fn new(height: usize, width: usize) -> OctetMatrix {
        let mut elements: Vec<Vec<u8>> = vec![];
        for _ in 0..height {
            elements.push(vec![0; width]);
        }
        OctetMatrix {
            height,
            width,
            elements
        }
    }

    pub fn mul_symbols(&self, symbols: &Vec<Symbol>) -> Vec<Symbol> {
        assert_eq!(self.width, symbols.len());
        assert_ne!(0, symbols.len());
        let mut result: Vec<Symbol> = vec![];
        for i in 0..self.height {
            let mut symbol = Symbol::zero(symbols[0].len());
            for j in 0..self.width {
                if self.elements[i][j] == 0 {
                    continue;
                }
                if self.elements[i][j] == 1 {
                   symbol += &symbols[j];
                }
                else {
                    symbol.fused_addassign_mul_scalar(&symbols[j], &Octet::new(self.elements[i][j]));
                }
            }
            result.push(symbol);
        }
        result
    }

    pub fn set(&mut self, i: usize, j: usize, value: Octet) {
        self.elements[i][j] = value.byte();
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn get(&self, i: usize, j: usize) -> Octet {
        Octet::new(self.elements[i][j])
    }

    pub fn swap_rows(&mut self, i: usize, j:usize) {
        self.elements.swap(i, j);
    }

    pub fn swap_columns(&mut self, i: usize, j:usize) {
        for row in 0..self.elements.len() {
            self.elements[row].swap(i, j);
        }
    }

    // Helper method for decoder phase 1
    // selects from [start_row, end_row) reading [start_col, end_col)
    // Returns (rows with two 1s, a row with two values > 1,
    // mapping from row number (offset by start_row) to number of non-zero values, "r" minimum positive number of non-zero values a row has)
    pub fn first_phase_selection(&self, start_row: usize, end_row: usize, start_col: usize, end_col: usize) -> (Vec<usize>, Option<usize>, Vec<u32>, Option<u32>) {
        let mut rows_with_two_ones = vec![];
        let mut row_with_two_greater_than_one = None;
        let mut non_zeros = vec![0; end_row - start_row];
        let mut r = std::u32::MAX;
        for row in start_row..end_row {
            let mut non_zero = 0;
            let mut ones = 0;
            for col in start_col..end_col {
                if self.elements[row][col] != 0 {
                    non_zero += 1;
                }
                if self.elements[row][col] != 1 {
                    ones += 1;
                }
            }
            if non_zero > 0 {
                non_zeros[row - start_row] = non_zero;
                if non_zero < r {
                    r = non_zero;
                }
            }
            if non_zero == 2 {
                if ones == 2 {
                    rows_with_two_ones.push(row);
                }
                else {
                    row_with_two_greater_than_one = Some(row);
                }
            }
        }

        if r < std::u32::MAX {
            (rows_with_two_ones, row_with_two_greater_than_one, non_zeros, Some(r))
        }
        else {
            (rows_with_two_ones, row_with_two_greater_than_one, non_zeros, None)
        }
    }

    // other must be a rows x rows matrix
    // sets self[0..rows][0..cols] = X * self[0..rows][0..cols]
    pub fn mul_assign_submatrix(&mut self, other: &OctetMatrix, rows: usize, cols: usize) {
        assert_eq!(rows, other.height());
        assert_eq!(rows, other.width());
        assert!(rows <= self.height());
        assert!(cols <= self.width());
        let temp = self.clone();
        for row in 0..rows {
            for col in 0..cols {
                let mut element = Octet::zero();
                for k in 0..rows {
                    unsafe {
                        element += Octet::new(*other.elements.get_unchecked(row).get_unchecked(k)) * Octet::new(*temp.elements.get_unchecked(k).get_unchecked(col));
                    }
                }
                unsafe {
                    *self.elements.get_unchecked_mut(row).get_unchecked_mut(col) = element.byte();
                }
            }
        }
    }

    pub fn fma_rows(&mut self, dest: usize, multiplicand: usize, scalar: &Octet) {
        // TODO: find a way to remove this clone()?
        let temp = self.elements[multiplicand].clone();
        if *scalar == Octet::one() {
            add_assign(&mut self.elements[dest], &temp);
        }
        else {
            fused_addassign_mul_scalar(&mut self.elements[dest], &temp, scalar);
        }
    }

    pub fn resize(&mut self, new_height: usize, new_width: usize) {
        assert!(new_height <= self.height);
        assert!(new_width <= self.width);
        let rows_to_discard = new_height..self.height;
        let cols_to_discard = new_width..self.width;
        self.elements.drain(rows_to_discard);
        for row in 0..self.elements.len() {
            self.elements[row].drain(cols_to_discard.clone());
        }
        self.height = new_height;
        self.width = new_width;
    }

    pub fn inverse(&self) -> Option<OctetMatrix> {
        // Calculate inverse using Gauss-Jordan elimination

        // Only implemented for square matrices
        assert_eq!(self.height, self.width);

        // Extend with identity on right side
        let mut intermediate = vec![];
        for i in 0..self.height {
            intermediate.push(vec![]);
            for j in 0..self.width {
                intermediate[i].push(Octet::new(self.elements[i][j]));
            }
        }
        for i in 0..self.height {
            for _ in 0..self.width {
                intermediate[i].push(Octet::zero());
            }
            intermediate[i][self.width + i] = Octet::one();
        }

        // Convert to row echelon form
        for i in 0..self.width {
            // Swap a row with leading coefficient i into place
            for j in i..self.height {
                if intermediate[j][i] != Octet::zero() {
                    intermediate.swap(i, j);
                    break;
                }
            }

            if intermediate[i][i] == Octet::zero() {
                // If all following rows are zero in this column, then matrix is singular
                return None
            }

            // Scale leading coefficient to 1
            if intermediate[i][i] != Octet::one() {
                let element_inverse = Octet::one() / intermediate[i][i].clone();
                for j in i..(2*self.width) {
                    intermediate[i][j] = intermediate[i][j].clone() * element_inverse.clone();
                }
            }

            // Zero out all following elements in i'th column
            for j in (i + 1)..self.height {
                if intermediate[j][i] != Octet::zero() {
                    let scalar = intermediate[j][i].clone();
                    // Multiply and subtract i'th row from j'th row
                    for k in i..(2*self.width) {
                        intermediate[j][k] = intermediate[j][k].clone() - scalar.clone() * intermediate[i][k].clone();
                    }
                }
            }
        }

        // Perform backwards elimination
        for i in (0..self.width).rev() {
            // Zero out all preceding elements in i'th column
            for j in 0..i {
                if intermediate[j][i] != Octet::zero() {
                    let scalar = intermediate[j][i].clone();
                    // Multiply and subtract i'th row from j'th row
                    for k in i..(2*self.width) {
                        intermediate[j][k] = intermediate[j][k].clone() - scalar.clone() * intermediate[i][k].clone();
                    }
                }
            }
        }

        // Return inverse
        let mut result = OctetMatrix::new(self.height, self.width);
        for row in 0..self.height {
            for column in 0..self.width {
                result.set(row, column, intermediate[row][column + self.width].clone());
            }
        }

        Some(result)
    }
}

impl Mul for OctetMatrix {
    type Output = OctetMatrix;

    fn mul(self, rhs: OctetMatrix) -> OctetMatrix {
        assert_eq!(self.width, rhs.height);
        let mut result = OctetMatrix::new(self.height, rhs.width);
        for i in 0..self.height {
            for j in 0..rhs.width {
                let mut element = Octet::zero();
                for k in 0..self.width {
                    element = element + self.get(i, k) * rhs.get(k, j);
                }
                result.set(i, j, element);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use matrix::tests::rand::Rng;
    use matrix::OctetMatrix;
    use symbol::Symbol;
    use octet::Octet;

    fn identity(size: usize) -> OctetMatrix {
        let mut result = OctetMatrix::new(size, size);
        for i in 0..size {
            result.set(i, i, Octet::one());
        }
        result
    }

    #[test]
    fn mul() {
        let identity = identity(4);
        let mut a = OctetMatrix::new(4, 5);
        for i in 0..4 {
            for j in 0..5 {
                a.set(i, j, Octet::new(rand::thread_rng().gen()));
            }
        }
        assert_eq!(a.clone(), identity * a.clone());
    }

    #[test]
    fn mul_symbol() {
        let identity = identity(4);
        let mut symbols: Vec<Symbol> = vec![];
        for _ in 0..4 {
            let mut data = vec![];
            for _ in 0..3 {
                data.push(rand::thread_rng().gen());
            }
            symbols.push(Symbol::new(data));
        }
        assert_eq!(symbols.clone(), identity.mul_symbols(&symbols));

        let mut a = OctetMatrix::new(4, 4);
        for i in 0..4 {
            for j in 0..4 {
                a.set(i, j, Octet::new(rand::thread_rng().gen()));
            }
        }
        // Statistically improbable that the random matrix, a, is the identity
        assert_ne!(symbols.clone(), a.mul_symbols(&symbols));
    }

    #[test]
    fn inverse() {
        let identity = identity(3);
        assert_eq!(identity, identity.clone() * identity.clone().inverse().unwrap());

        let mut a = OctetMatrix::new(3, 3);
        a.set(0, 0, Octet::new(1));
        a.set(0, 1, Octet::new(2));
        a.set(0, 2, Octet::new(3));

        a.set(1, 0, Octet::new(4));
        a.set(1, 1, Octet::new(5));
        a.set(1, 2, Octet::new(6));

        a.set(2, 0, Octet::new(7));
        a.set(2, 1, Octet::new(8));
        a.set(2, 2, Octet::new(9));
        assert_eq!(identity, a.clone() * a.clone().inverse().unwrap());
    }
}
