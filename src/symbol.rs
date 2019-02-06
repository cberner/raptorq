use octet::Octet;
use octet::OCTET_MUL;
use std::ops::AddAssign;

#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    value: Vec<u8>
}

impl Symbol {
    pub fn new(value: Vec<u8>) -> Symbol {
        Symbol {
            value
        }
    }

    pub fn zero(size: usize) -> Symbol {
        Symbol {
            value: vec![0; size]
        }
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.value.clone()
    }

    pub fn mulassign_scalar(&mut self, scalar: &Octet) {
        unsafe {
            assert_ne!(0, OCTET_MUL[1 << 8 | 1], "Must call Octet::static_init()");
        }
        let scalar_index = (scalar.byte() as usize) << 8;
        for i in 0..self.value.len() {
            unsafe {
                *self.value.get_unchecked_mut(i) = *OCTET_MUL.get_unchecked(scalar_index + *self.value.get_unchecked(i) as usize);
            }
        }
    }

    pub fn fused_addassign_mul_scalar(&mut self, other: &Symbol, scalar: &Octet) {
        // TODO: enable these in debug only?
        assert_ne!(*scalar, Octet::one(), "Don't call this with one. Use += instead");
        assert_ne!(*scalar, Octet::zero(), "Don't call with zero. It's very inefficient");

        unsafe {
            assert_ne!(0, OCTET_MUL[1 << 8 | 1], "Must call Octet::static_init()");
        }

        assert_eq!(self.value.len(), other.value.len());
        let scalar_index = (scalar.byte() as usize) << 8;
        for i in 0..self.value.len() {
            unsafe  {
                *self.value.get_unchecked_mut(i) ^= *OCTET_MUL.get_unchecked(scalar_index + *other.value.get_unchecked(i) as usize);
            }
        }
    }
}

impl<'a> Symbol {
    fn add_assign_fallback(&mut self, other: &'a Symbol) {
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
                *self.value.get_unchecked_mut(i) ^= other.value.get_unchecked(i);
            }
        }
    }

    fn add_assign_avx2(&mut self, other: &'a Symbol) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        assert_eq!(self.value.len(), other.value.len());
        let self_avx_ptr = self.value.as_mut_ptr() as *mut __m256i;
        let other_avx_ptr = other.value.as_ptr() as *const __m256i;
        for i in 0..(self.value.len() / 32) {
            unsafe {
                let self_vec = _mm256_loadu_si256(self_avx_ptr.add(i));
                let other_vec = _mm256_loadu_si256(other_avx_ptr.add(i));
                let result = _mm256_xor_si256(self_vec, other_vec);
                _mm256_storeu_si256(self_avx_ptr.add(i), result);
            }
        }

        let remainder = self.value.len() % 32;
        let self_ptr = self.value.as_mut_ptr() as *mut u64;
        let other_ptr = other.value.as_ptr() as *const u64;
        for i in ((self.value.len() - remainder) / 8)..(self.value.len() / 8) {
            unsafe {
                *self_ptr.add(i) ^= *other_ptr.add(i);
            }
        }

        let remainder = self.value.len() % 8;
        for i in (self.value.len() - remainder)..self.value.len() {
            unsafe {
                *self.value.get_unchecked_mut(i) ^= other.value.get_unchecked(i);
            }
        }
    }
}


impl<'a> AddAssign<&'a Symbol> for Symbol {
    fn add_assign(&mut self, other: &'a Symbol) {
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
        return self.add_assign_avx2(other);

        self.add_assign_fallback(other);
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use symbol::tests::rand::Rng;
    use symbol::Symbol;

    #[test]
    fn add_assign() {
        let symbol_size = 41;
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
