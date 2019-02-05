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
        let self_ptr = self.value.as_mut_ptr() as *mut u64;
        let other_ptr = other.value.as_ptr() as *const u64;
        for i in 0..(self.value.len() / 8) {
            unsafe {
                *self_ptr.add(i) ^= *other_ptr.add(i);
            }
        }
        let remainder = self.value.len() % 8;
        for i in (self.value.len() - remainder)..self.value.len() {
            unsafe {
                *self.value.get_unchecked_mut(i) += other.value.get_unchecked(i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use symbol::tests::rand::Rng;
    use symbol::Symbol;

    #[test]
    fn add_assign() {
        let symbol_size = 17;
        let mut data1: Vec<u8> = vec![0; symbol_size];
        let mut data2: Vec<u8> = vec![0; symbol_size];
        let mut result: Vec<u8> = vec![0; symbol_size];
        for i in 0..symbol_size {
            data1[i] = rand::thread_rng().gen();
            data2[i] = rand::thread_rng().gen();
            result[i] = data1[i] ^ data2[i];
        }
        let mut symbol1 = Symbol::new(data1);
        let symbol2 = Symbol::new(data2);

        symbol1 += &symbol2;
        assert_eq!(result, symbol1.bytes());
    }
}
