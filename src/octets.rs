use crate::octet::Octet;
use crate::octet::OCTET_MUL;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::octet::OCTET_MUL_HI_BITS;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::octet::OCTET_MUL_LOW_BITS;

// An octet vec containing only binary values, which are bit-packed for efficiency
pub struct BinaryOctetVec {
    // Values are stored packed into the highest bits, with the last value at the highest bit of the
    // last byte. Therefore, there may be trailing bits (least significant) which are unused
    elements: Vec<u64>,
    length: usize,
}

impl BinaryOctetVec {
    pub(crate) const WORD_WIDTH: usize = 64;

    pub fn new(elements: Vec<u64>, length: usize) -> Self {
        assert_eq!(
            elements.len(),
            (length + Self::WORD_WIDTH - 1) / Self::WORD_WIDTH
        );
        BinaryOctetVec { elements, length }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    fn to_octet_vec(&self) -> Vec<u8> {
        let mut word = 0;
        let mut bit = self.padding_bits();

        let result = (0..self.length)
            .map(|_| {
                let value = if self.elements[word] & BinaryOctetVec::select_mask(bit) == 0 {
                    0
                } else {
                    1
                };

                bit += 1;
                if bit == 64 {
                    word += 1;
                    bit = 0;
                }

                value
            })
            .collect();
        assert_eq!(word, self.elements.len());
        assert_eq!(bit, 0);
        result
    }

    pub fn padding_bits(&self) -> usize {
        (BinaryOctetVec::WORD_WIDTH - (self.length % BinaryOctetVec::WORD_WIDTH))
            % BinaryOctetVec::WORD_WIDTH
    }

    pub fn select_mask(bit: usize) -> u64 {
        1u64 << (bit as u64)
    }
}

pub fn fused_addassign_mul_scalar_binary(
    octets: &mut [u8],
    other: &BinaryOctetVec,
    scalar: &Octet,
) {
    debug_assert_ne!(
        *scalar,
        Octet::zero(),
        "Don't call with zero. It's very inefficient"
    );

    assert_eq!(octets.len(), other.len());
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("bmi1") {
            unsafe {
                return fused_addassign_mul_scalar_binary_avx2(octets, other, scalar);
            }
        }
    }

    // TODO: write an optimized fallback that does call .to_octet_vec()
    return fused_addassign_mul_scalar(octets, &other.to_octet_vec(), scalar);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[target_feature(enable = "bmi1")]
unsafe fn fused_addassign_mul_scalar_binary_avx2(
    octets: &mut [u8],
    other: &BinaryOctetVec,
    scalar: &Octet,
) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let first_bit = other.padding_bits();
    let other_u32 = std::slice::from_raw_parts(
        other.elements.as_ptr() as *const u32,
        other.elements.len() * 2,
    );
    let mut other_batch_start_index = first_bit / 32;
    let first_bits = other_u32[other_batch_start_index];
    let bit_in_first_bits = first_bit % 32;
    let mut remaining = octets.len();
    let mut self_avx_ptr = octets.as_mut_ptr();
    // Handle first bits to make remainder 32bit aligned
    if bit_in_first_bits > 0 {
        let control = bit_in_first_bits as u32 | 0x100;
        for (i, val) in octets.iter_mut().enumerate().take(32 - bit_in_first_bits) {
            let other_byte = _bextr2_u32(first_bits, control + i as u32) as u8;
            // other_byte is binary, so u8 multiplication is the same as GF256 multiplication
            *val ^= scalar.byte() * other_byte;
        }
        remaining -= 32 - bit_in_first_bits;
        other_batch_start_index += 1;
        self_avx_ptr = self_avx_ptr.add(32 - bit_in_first_bits);
    }

    assert_eq!(remaining % 32, 0);

    // See: https://stackoverflow.com/questions/24225786/fastest-way-to-unpack-32-bits-to-a-32-byte-simd-vector
    let shuffle_mask = _mm256_set_epi64x(
        0x03030303_03030303,
        0x02020202_02020202,
        0x01010101_01010101,
        0,
    );
    let bit_select_mask = _mm256_set1_epi64x(0x80402010_08040201u64 as i64);
    let scalar_avx = _mm256_set1_epi8(scalar.byte() as i8);
    // Process the rest in 256bit chunks
    for i in 0..(remaining / 32) {
        // Convert from bit packed u32 to 32xu8
        let other_vec = _mm256_set1_epi32(other_u32[other_batch_start_index + i] as i32);
        let other_vec = _mm256_shuffle_epi8(other_vec, shuffle_mask);
        let other_vec = _mm256_andnot_si256(other_vec, bit_select_mask);
        // The bits are now unpacked, but aren't in a defined position (may be in 0-7 bit of each byte),
        // and are inverted (due to AND NOT)

        // Test against zero to get non-inverted selection
        let other_vec = _mm256_cmpeq_epi8(other_vec, _mm256_setzero_si256());
        // Multiply by scalar. other_vec is binary (0xFF or 0x00), so just mask with the scalar
        let product = _mm256_and_si256(other_vec, scalar_avx);

        // Add to self
        #[allow(clippy::cast_ptr_alignment)]
        let self_vec = _mm256_loadu_si256((self_avx_ptr as *const __m256i).add(i));
        let result = _mm256_xor_si256(self_vec, product);
        #[allow(clippy::cast_ptr_alignment)]
        _mm256_storeu_si256((self_avx_ptr as *mut __m256i).add(i), result);
    }
}

fn mulassign_scalar_fallback(octets: &mut [u8], scalar: &Octet) {
    let scalar_index = usize::from(scalar.byte());
    for item in octets {
        let octet_index = usize::from(*item);
        // SAFETY: `OCTET_MUL` is a 256x256 matrix, both indexes are `u8` inputs.
        *item = unsafe {
            *OCTET_MUL
                .get_unchecked(scalar_index)
                .get_unchecked(octet_index)
        };
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn mulassign_scalar_avx2(octets: &mut [u8], scalar: &Octet) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let low_mask = _mm256_set1_epi8(0x0F);
    let hi_mask = _mm256_set1_epi8(0xF0 as u8 as i8);
    let self_avx_ptr = octets.as_mut_ptr();
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let low_table =
        _mm256_loadu_si256(OCTET_MUL_LOW_BITS[scalar.byte() as usize].as_ptr() as *const __m256i);
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let hi_table =
        _mm256_loadu_si256(OCTET_MUL_HI_BITS[scalar.byte() as usize].as_ptr() as *const __m256i);

    for i in 0..(octets.len() / 32) {
        #[allow(clippy::cast_ptr_alignment)]
        let self_vec = _mm256_loadu_si256((self_avx_ptr as *const __m256i).add(i));
        let low = _mm256_and_si256(self_vec, low_mask);
        let low_result = _mm256_shuffle_epi8(low_table, low);
        let hi = _mm256_and_si256(self_vec, hi_mask);
        let hi = _mm256_srli_epi64(hi, 4);
        let hi_result = _mm256_shuffle_epi8(hi_table, hi);
        let result = _mm256_xor_si256(hi_result, low_result);
        #[allow(clippy::cast_ptr_alignment)]
        _mm256_storeu_si256((self_avx_ptr as *mut __m256i).add(i), result);
    }

    let remainder = octets.len() % 32;
    let scalar_index = scalar.byte() as usize;
    for i in (octets.len() - remainder)..octets.len() {
        *octets.get_unchecked_mut(i) = *OCTET_MUL
            .get_unchecked(scalar_index)
            .get_unchecked(*octets.get_unchecked(i) as usize);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "ssse3")]
unsafe fn mulassign_scalar_ssse3(octets: &mut [u8], scalar: &Octet) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let low_mask = _mm_set1_epi8(0x0F);
    let hi_mask = _mm_set1_epi8(0xF0 as u8 as i8);
    let self_ssse_ptr = octets.as_mut_ptr();
    #[allow(clippy::cast_ptr_alignment)]
    let low_table =
        _mm_loadu_si128(OCTET_MUL_LOW_BITS[scalar.byte() as usize].as_ptr() as *const __m128i);
    #[allow(clippy::cast_ptr_alignment)]
    let hi_table =
        _mm_loadu_si128(OCTET_MUL_HI_BITS[scalar.byte() as usize].as_ptr() as *const __m128i);

    for i in 0..(octets.len() / 16) {
        #[allow(clippy::cast_ptr_alignment)]
        let self_vec = _mm_loadu_si128((self_ssse_ptr as *const __m128i).add(i));
        let low = _mm_and_si128(self_vec, low_mask);
        let low_result = _mm_shuffle_epi8(low_table, low);
        let hi = _mm_and_si128(self_vec, hi_mask);
        let hi = _mm_srli_epi64(hi, 4);
        let hi_result = _mm_shuffle_epi8(hi_table, hi);
        let result = _mm_xor_si128(hi_result, low_result);
        #[allow(clippy::cast_ptr_alignment)]
        _mm_storeu_si128((self_ssse_ptr as *mut __m128i).add(i), result);
    }

    let remainder = octets.len() % 16;
    let scalar_index = scalar.byte() as usize;
    for i in (octets.len() - remainder)..octets.len() {
        *octets.get_unchecked_mut(i) = *OCTET_MUL
            .get_unchecked(scalar_index)
            .get_unchecked(*octets.get_unchecked(i) as usize);
    }
}

pub fn mulassign_scalar(octets: &mut [u8], scalar: &Octet) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return mulassign_scalar_avx2(octets, scalar);
            }
        }
        if is_x86_feature_detected!("ssse3") {
            unsafe {
                return mulassign_scalar_ssse3(octets, scalar);
            }
        }
    }

    return mulassign_scalar_fallback(octets, scalar);
}

fn fused_addassign_mul_scalar_fallback(octets: &mut [u8], other: &[u8], scalar: &Octet) {
    let scalar_index = scalar.byte() as usize;
    for i in 0..octets.len() {
        unsafe {
            *octets.get_unchecked_mut(i) ^= *OCTET_MUL
                .get_unchecked(scalar_index)
                .get_unchecked(*other.get_unchecked(i) as usize);
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn fused_addassign_mul_scalar_avx2(octets: &mut [u8], other: &[u8], scalar: &Octet) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let low_mask = _mm256_set1_epi8(0x0F);
    let hi_mask = _mm256_set1_epi8(0xF0 as u8 as i8);
    let self_avx_ptr = octets.as_mut_ptr();
    let other_avx_ptr = other.as_ptr();
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let low_table =
        _mm256_loadu_si256(OCTET_MUL_LOW_BITS[scalar.byte() as usize].as_ptr() as *const __m256i);
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let hi_table =
        _mm256_loadu_si256(OCTET_MUL_HI_BITS[scalar.byte() as usize].as_ptr() as *const __m256i);

    for i in 0..(octets.len() / 32) {
        // Multiply by scalar
        #[allow(clippy::cast_ptr_alignment)]
        let other_vec = _mm256_loadu_si256((other_avx_ptr as *const __m256i).add(i));
        let low = _mm256_and_si256(other_vec, low_mask);
        let low_result = _mm256_shuffle_epi8(low_table, low);
        let hi = _mm256_and_si256(other_vec, hi_mask);
        let hi = _mm256_srli_epi64(hi, 4);
        let hi_result = _mm256_shuffle_epi8(hi_table, hi);
        let other_vec = _mm256_xor_si256(hi_result, low_result);

        // Add to self
        #[allow(clippy::cast_ptr_alignment)]
        let self_vec = _mm256_loadu_si256((self_avx_ptr as *const __m256i).add(i));
        let result = _mm256_xor_si256(self_vec, other_vec);
        #[allow(clippy::cast_ptr_alignment)]
        _mm256_storeu_si256((self_avx_ptr as *mut __m256i).add(i), result);
    }

    let remainder = octets.len() % 32;
    let scalar_index = scalar.byte() as usize;
    for i in (octets.len() - remainder)..octets.len() {
        *octets.get_unchecked_mut(i) ^= *OCTET_MUL
            .get_unchecked(scalar_index)
            .get_unchecked(*other.get_unchecked(i) as usize);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "ssse3")]
unsafe fn fused_addassign_mul_scalar_ssse3(octets: &mut [u8], other: &[u8], scalar: &Octet) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let low_mask = _mm_set1_epi8(0x0F);
    let hi_mask = _mm_set1_epi8(0xF0 as u8 as i8);
    let self_ssse_ptr = octets.as_mut_ptr();
    let other_ssse_ptr = other.as_ptr();
    #[allow(clippy::cast_ptr_alignment)]
    let low_table =
        _mm_loadu_si128(OCTET_MUL_LOW_BITS[scalar.byte() as usize].as_ptr() as *const __m128i);
    #[allow(clippy::cast_ptr_alignment)]
    let hi_table =
        _mm_loadu_si128(OCTET_MUL_HI_BITS[scalar.byte() as usize].as_ptr() as *const __m128i);

    for i in 0..(octets.len() / 16) {
        // Multiply by scalar
        #[allow(clippy::cast_ptr_alignment)]
        let other_vec = _mm_loadu_si128((other_ssse_ptr as *const __m128i).add(i));
        let low = _mm_and_si128(other_vec, low_mask);
        let low_result = _mm_shuffle_epi8(low_table, low);
        let hi = _mm_and_si128(other_vec, hi_mask);
        let hi = _mm_srli_epi64(hi, 4);
        let hi_result = _mm_shuffle_epi8(hi_table, hi);
        let other_vec = _mm_xor_si128(hi_result, low_result);

        // Add to self
        #[allow(clippy::cast_ptr_alignment)]
        let self_vec = _mm_loadu_si128((self_ssse_ptr as *const __m128i).add(i));
        let result = _mm_xor_si128(self_vec, other_vec);
        #[allow(clippy::cast_ptr_alignment)]
        _mm_storeu_si128((self_ssse_ptr as *mut __m128i).add(i), result);
    }

    let remainder = octets.len() % 16;
    let scalar_index = scalar.byte() as usize;
    for i in (octets.len() - remainder)..octets.len() {
        *octets.get_unchecked_mut(i) ^= *OCTET_MUL
            .get_unchecked(scalar_index)
            .get_unchecked(*other.get_unchecked(i) as usize);
    }
}

pub fn fused_addassign_mul_scalar(octets: &mut [u8], other: &[u8], scalar: &Octet) {
    debug_assert_ne!(
        *scalar,
        Octet::one(),
        "Don't call this with one. Use += instead"
    );
    debug_assert_ne!(
        *scalar,
        Octet::zero(),
        "Don't call with zero. It's very inefficient"
    );

    assert_eq!(octets.len(), other.len());
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return fused_addassign_mul_scalar_avx2(octets, other, scalar);
            }
        }
        if is_x86_feature_detected!("ssse3") {
            unsafe {
                return fused_addassign_mul_scalar_ssse3(octets, other, scalar);
            }
        }
    }

    return fused_addassign_mul_scalar_fallback(octets, other, scalar);
}

fn add_assign_fallback(octets: &mut [u8], other: &[u8]) {
    assert_eq!(octets.len(), other.len());
    let self_ptr = octets.as_mut_ptr();
    let other_ptr = other.as_ptr();
    for i in 0..(octets.len() / 8) {
        unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            let self_value = (self_ptr as *const u64).add(i).read_unaligned();
            #[allow(clippy::cast_ptr_alignment)]
            let other_value = (other_ptr as *const u64).add(i).read_unaligned();
            let result = self_value ^ other_value;
            #[allow(clippy::cast_ptr_alignment)]
            (self_ptr as *mut u64).add(i).write_unaligned(result);
        }
    }
    let remainder = octets.len() % 8;
    for i in (octets.len() - remainder)..octets.len() {
        unsafe {
            *octets.get_unchecked_mut(i) ^= other.get_unchecked(i);
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn add_assign_avx2(octets: &mut [u8], other: &[u8]) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    assert_eq!(octets.len(), other.len());
    let self_avx_ptr = octets.as_mut_ptr();
    let other_avx_ptr = other.as_ptr();
    for i in 0..(octets.len() / 32) {
        #[allow(clippy::cast_ptr_alignment)]
        let self_vec = _mm256_loadu_si256((self_avx_ptr as *const __m256i).add(i));
        #[allow(clippy::cast_ptr_alignment)]
        let other_vec = _mm256_loadu_si256((other_avx_ptr as *const __m256i).add(i));
        let result = _mm256_xor_si256(self_vec, other_vec);
        #[allow(clippy::cast_ptr_alignment)]
        _mm256_storeu_si256((self_avx_ptr as *mut __m256i).add(i), result);
    }

    let remainder = octets.len() % 32;
    let self_ptr = octets.as_mut_ptr();
    let other_ptr = other.as_ptr();
    for i in ((octets.len() - remainder) / 8)..(octets.len() / 8) {
        #[allow(clippy::cast_ptr_alignment)]
        let self_value = (self_ptr as *mut u64).add(i).read_unaligned();
        #[allow(clippy::cast_ptr_alignment)]
        let other_value = (other_ptr as *mut u64).add(i).read_unaligned();
        let result = self_value ^ other_value;
        #[allow(clippy::cast_ptr_alignment)]
        (self_ptr as *mut u64).add(i).write_unaligned(result);
    }

    let remainder = octets.len() % 8;
    for i in (octets.len() - remainder)..octets.len() {
        *octets.get_unchecked_mut(i) ^= other.get_unchecked(i);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "ssse3")]
unsafe fn add_assign_ssse3(octets: &mut [u8], other: &[u8]) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    assert_eq!(octets.len(), other.len());
    let self_ssse_ptr = octets.as_mut_ptr();
    let other_ssse_ptr = other.as_ptr();
    for i in 0..(octets.len() / 16) {
        #[allow(clippy::cast_ptr_alignment)]
        let self_vec = _mm_loadu_si128((self_ssse_ptr as *const __m128i).add(i));
        #[allow(clippy::cast_ptr_alignment)]
        let other_vec = _mm_loadu_si128((other_ssse_ptr as *const __m128i).add(i));
        let result = _mm_xor_si128(self_vec, other_vec);
        #[allow(clippy::cast_ptr_alignment)]
        _mm_storeu_si128((self_ssse_ptr as *mut __m128i).add(i), result);
    }

    let remainder = octets.len() % 16;
    let self_ptr = octets.as_mut_ptr();
    let other_ptr = other.as_ptr();
    for i in ((octets.len() - remainder) / 8)..(octets.len() / 8) {
        #[allow(clippy::cast_ptr_alignment)]
        let self_value = (self_ptr as *mut u64).add(i).read_unaligned();
        #[allow(clippy::cast_ptr_alignment)]
        let other_value = (other_ptr as *mut u64).add(i).read_unaligned();
        let result = self_value ^ other_value;
        #[allow(clippy::cast_ptr_alignment)]
        (self_ptr as *mut u64).add(i).write_unaligned(result);
    }

    let remainder = octets.len() % 8;
    for i in (octets.len() - remainder)..octets.len() {
        *octets.get_unchecked_mut(i) ^= other.get_unchecked(i);
    }
}

pub fn add_assign(octets: &mut [u8], other: &[u8]) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return add_assign_avx2(octets, other);
            }
        }
        if is_x86_feature_detected!("ssse3") {
            unsafe {
                return add_assign_ssse3(octets, other);
            }
        }
    }

    return add_assign_fallback(octets, other);
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::octet::Octet;
    use crate::octets::fused_addassign_mul_scalar;
    use crate::octets::mulassign_scalar;

    #[test]
    fn mul_assign() {
        let size = 41;
        let scalar = Octet::new(rand::thread_rng().gen_range(1, 255));
        let mut data1: Vec<u8> = vec![0; size];
        let mut expected: Vec<u8> = vec![0; size];
        for i in 0..size {
            data1[i] = rand::thread_rng().gen();
            expected[i] = (&Octet::new(data1[i]) * &scalar).byte();
        }

        mulassign_scalar(&mut data1, &scalar);

        assert_eq!(expected, data1);
    }

    #[test]
    fn fma() {
        let size = 41;
        let scalar = Octet::new(rand::thread_rng().gen_range(1, 255));
        let mut data1: Vec<u8> = vec![0; size];
        let mut data2: Vec<u8> = vec![0; size];
        let mut expected: Vec<u8> = vec![0; size];
        for i in 0..size {
            data1[i] = rand::thread_rng().gen();
            data2[i] = rand::thread_rng().gen();
            expected[i] = (Octet::new(data1[i]) + &Octet::new(data2[i]) * &scalar).byte();
        }

        fused_addassign_mul_scalar(&mut data1, &data2, &scalar);

        assert_eq!(expected, data1);
    }
}
