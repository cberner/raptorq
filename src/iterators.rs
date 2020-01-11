use crate::octet::Octet;
use crate::sparse_vec::{SparseBinaryVec, SparseValuelessVec};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct KeyIter {
    sparse: bool,
    dense_index: usize,
    dense_end: usize,
    sparse_rows: Option<Vec<usize>>,
}

impl Iterator for KeyIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sparse {
            return self.sparse_rows.as_mut().unwrap().pop();
        } else if self.dense_index == self.dense_end {
            return None;
        } else {
            let old_index = self.dense_index;
            self.dense_index += 1;
            return Some(old_index);
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct BorrowedKeyIter<'a> {
    sparse: bool,
    dense_index: usize,
    dense_end: usize,
    sparse_rows: Option<&'a SparseValuelessVec>,
    sparse_start_row: usize,
    sparse_end_row: usize,
    sparse_index: usize,
    physical_row_to_logical: Option<&'a [usize]>,
}

impl<'a> BorrowedKeyIter<'a> {
    pub fn new_sparse(
        sparse_rows: &'a SparseValuelessVec,
        sparse_start_row: usize,
        sparse_end_row: usize,
        physical_row_to_logical: &'a [usize],
    ) -> BorrowedKeyIter<'a> {
        BorrowedKeyIter {
            sparse: true,
            dense_index: 0,
            dense_end: 0,
            sparse_rows: Some(sparse_rows),
            sparse_start_row,
            sparse_end_row,
            sparse_index: 0,
            physical_row_to_logical: Some(physical_row_to_logical),
        }
    }

    pub fn new_dense(dense_index: usize, dense_end: usize) -> BorrowedKeyIter<'a> {
        BorrowedKeyIter {
            sparse: false,
            dense_index,
            dense_end,
            sparse_rows: None,
            sparse_start_row: 0,
            sparse_end_row: 0,
            sparse_index: 0,
            physical_row_to_logical: None,
        }
    }

    pub fn clone(&self) -> KeyIter {
        // Convert to logical indices, since ClonedOctetIter doesn't handle physical
        let sparse_rows = self.sparse_rows.map(|x| {
            x.keys()
                .map(|physical_row| self.physical_row_to_logical.unwrap()[*physical_row])
                .filter(|logical_row| {
                    *logical_row >= self.sparse_start_row && *logical_row < self.sparse_end_row
                })
                .collect()
        });
        KeyIter {
            sparse: self.sparse,
            dense_index: self.dense_index,
            dense_end: self.dense_end,
            sparse_rows,
        }
    }
}

impl<'a> Iterator for BorrowedKeyIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sparse {
            let elements = self.sparse_rows.unwrap();
            // Need to iterate over the whole array, since they're not sorted by logical row
            while self.sparse_index < elements.len() {
                let physical_row = elements.get_by_raw_index(self.sparse_index);
                self.sparse_index += 1;
                let logical_row = self.physical_row_to_logical.unwrap()[*physical_row];
                if logical_row >= self.sparse_start_row && logical_row < self.sparse_end_row {
                    return Some(logical_row);
                }
            }
            return None;
        } else if self.dense_index == self.dense_end {
            return None;
        } else {
            let old_index = self.dense_index;
            self.dense_index += 1;
            return Some(old_index);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ClonedOctetIter {
    sparse: bool,
    end_col: usize,
    dense_elements: Option<Vec<u8>>,
    dense_index: usize,
    sparse_elements: Option<Vec<(usize, Octet)>>,
    sparse_index: usize,
}

impl Iterator for ClonedOctetIter {
    type Item = (usize, Octet);

    fn next(&mut self) -> Option<Self::Item> {
        if self.sparse {
            let elements = self.sparse_elements.as_ref().unwrap();
            if self.sparse_index == elements.len() {
                return None;
            } else {
                let old_index = self.sparse_index;
                self.sparse_index += 1;
                return Some(elements[old_index].clone());
            }
        } else if self.dense_index == self.end_col {
            return None;
        } else {
            let old_index = self.dense_index;
            self.dense_index += 1;
            return Some((
                old_index,
                Octet::new(self.dense_elements.as_ref().unwrap()[old_index]),
            ));
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct OctetIter<'a> {
    sparse: bool,
    start_col: usize,
    end_col: usize,
    dense_elements: Option<&'a Vec<u8>>,
    dense_index: usize,
    sparse_elements: Option<&'a SparseBinaryVec>,
    sparse_index: usize,
    sparse_physical_col_to_logical: Option<&'a [usize]>,
}

impl<'a> OctetIter<'a> {
    pub fn new_sparse(
        start_col: usize,
        end_col: usize,
        sparse_elements: &'a SparseBinaryVec,
        sparse_physical_col_to_logical: &'a [usize],
    ) -> OctetIter<'a> {
        OctetIter {
            sparse: true,
            start_col,
            end_col,
            dense_elements: None,
            dense_index: 0,
            sparse_elements: Some(sparse_elements),
            sparse_index: 0,
            sparse_physical_col_to_logical: Some(sparse_physical_col_to_logical),
        }
    }

    #[allow(clippy::ptr_arg)]
    pub fn new_dense(
        start_col: usize,
        end_col: usize,
        dense_elements: &'a Vec<u8>,
        dense_index: usize,
    ) -> OctetIter<'a> {
        OctetIter {
            sparse: false,
            start_col,
            end_col,
            dense_elements: Some(dense_elements),
            dense_index,
            sparse_elements: None,
            sparse_index: 0,
            sparse_physical_col_to_logical: None,
        }
    }

    pub fn clone(&self) -> ClonedOctetIter {
        // Convert to logical indices, since ClonedOctetIter doesn't handle physical
        let sparse_elements = self.sparse_elements.map(|x| {
            x.keys_values()
                .map(|(physical_col, value)| {
                    (
                        self.sparse_physical_col_to_logical.unwrap()[*physical_col],
                        value.clone(),
                    )
                })
                .filter(|(logical_col, _)| {
                    *logical_col >= self.start_col && *logical_col < self.end_col
                })
                .collect()
        });
        ClonedOctetIter {
            sparse: self.sparse,
            end_col: self.end_col,
            dense_elements: self.dense_elements.cloned(),
            dense_index: self.dense_index,
            sparse_elements,
            sparse_index: self.sparse_index,
        }
    }
}

impl<'a> Iterator for OctetIter<'a> {
    type Item = (usize, Octet);

    fn next(&mut self) -> Option<Self::Item> {
        if self.sparse {
            let elements = self.sparse_elements.unwrap();
            // Need to iterate over the whole array, since they're not sorted by logical col
            if self.sparse_index >= elements.len() {
                return None;
            } else {
                while self.sparse_index < elements.len() {
                    let entry = elements.get_by_raw_index(self.sparse_index);
                    self.sparse_index += 1;
                    let logical_col = self.sparse_physical_col_to_logical.unwrap()[entry.0];
                    if logical_col >= self.start_col && logical_col < self.end_col {
                        return Some((logical_col, entry.1.clone()));
                    }
                }
                return None;
            }
        } else if self.dense_index == self.end_col {
            return None;
        } else {
            let old_index = self.dense_index;
            self.dense_index += 1;
            return Some((
                old_index,
                Octet::new(self.dense_elements.unwrap()[old_index]),
            ));
        }
    }
}
