use octet::Octet;
use std::ops::AddAssign;

#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    value: Vec<Octet>
}

impl Symbol {
    pub fn new(value: Vec<u8>) -> Symbol {
        Symbol {
            value: value.iter().map(|&x| Octet::from(x)).collect()
        }
    }

    pub fn zero(size: usize) -> Symbol {
        Symbol {
            value: vec![Octet::zero(); size]
        }
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.value.iter().map(|octet| octet.clone().into()).collect()
    }

    pub fn mulassign_scalar(&mut self, scalar: &Octet) {
        for i in 0..self.value.len() {
            unsafe {
                *self.value.get_unchecked_mut(i) = self.value.get_unchecked(i) * scalar;
            }
        }
    }

    pub fn fused_addassign_mul_scalar(&mut self, other: &Symbol, scalar: &Octet) {
        // TODO: enable these in debug only?
        assert_ne!(*scalar, Octet::one(), "Don't call this with one. Use += instead");
        assert_ne!(*scalar, Octet::zero(), "Don't call with zero. It's very inefficient");

        assert_eq!(self.value.len(), other.value.len());
        for i in 0..self.value.len() {
            unsafe  {
                self.value.get_unchecked_mut(i).fma(other.value.get_unchecked(i), &scalar);
            }
        }
    }
}

impl<'a> AddAssign<&'a Symbol> for Symbol {
    fn add_assign(&mut self, other: &'a Symbol) {
        assert_eq!(self.value.len(), other.value.len());
        for i in 0..self.value.len() {
            unsafe {
                *self.value.get_unchecked_mut(i) += other.value.get_unchecked(i);
            }
        }
    }
}
