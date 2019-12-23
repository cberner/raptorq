use crate::octet::Octet;
use crate::octet::OCTET_MUL;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::octet::OCTET_MUL_HI_BITS;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::octet::OCTET_MUL_LOW_BITS;

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
    // Safe because _mm256_loadu_si256 loads from unaligned memory, and _mm256_storeu_si256
    // stores to unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let self_avx_ptr = octets.as_mut_ptr() as *mut __m256i;
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let low_table =
        _mm256_loadu_si256(OCTET_MUL_LOW_BITS[scalar.byte() as usize].as_ptr() as *const __m256i);
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let hi_table =
        _mm256_loadu_si256(OCTET_MUL_HI_BITS[scalar.byte() as usize].as_ptr() as *const __m256i);

    for i in 0..(octets.len() / 32) {
        let self_vec = _mm256_loadu_si256(self_avx_ptr.add(i));
        let low = _mm256_and_si256(self_vec, low_mask);
        let low_result = _mm256_shuffle_epi8(low_table, low);
        let hi = _mm256_and_si256(self_vec, hi_mask);
        let hi = _mm256_srli_epi64(hi, 4);
        let hi_result = _mm256_shuffle_epi8(hi_table, hi);
        let result = _mm256_xor_si256(hi_result, low_result);
        _mm256_storeu_si256(self_avx_ptr.add(i), result);
    }

    let remainder = octets.len() % 32;
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
    // Safe because _mm256_loadu_si256 loads from unaligned memory, and _mm256_storeu_si256
    // stores to unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let self_avx_ptr = octets.as_mut_ptr() as *mut __m256i;
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let other_avx_ptr = other.as_ptr() as *const __m256i;
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
        let other_vec = _mm256_loadu_si256(other_avx_ptr.add(i));
        let low = _mm256_and_si256(other_vec, low_mask);
        let low_result = _mm256_shuffle_epi8(low_table, low);
        let hi = _mm256_and_si256(other_vec, hi_mask);
        let hi = _mm256_srli_epi64(hi, 4);
        let hi_result = _mm256_shuffle_epi8(hi_table, hi);
        let other_vec = _mm256_xor_si256(hi_result, low_result);

        // Add to self
        let self_vec = _mm256_loadu_si256(self_avx_ptr.add(i));
        let result = _mm256_xor_si256(self_vec, other_vec);
        _mm256_storeu_si256(self_avx_ptr.add(i), result);
    }

    let remainder = octets.len() % 32;
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
    }

    return fused_addassign_mul_scalar_fallback(octets, other, scalar);
}

fn add_assign_fallback(octets: &mut [u8], other: &[u8]) {
    assert_eq!(octets.len(), other.len());
    let self_ptr = octets.as_mut_ptr() as *mut u64;
    let other_ptr = other.as_ptr() as *const u64;
    for i in 0..(octets.len() / 8) {
        unsafe {
            *self_ptr.add(i) ^= *other_ptr.add(i);
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
    // Safe because _mm256_loadu_si256 loads from unaligned memory, and _mm256_storeu_si256
    // stores to unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let self_avx_ptr = octets.as_mut_ptr() as *mut __m256i;
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let other_avx_ptr = other.as_ptr() as *const __m256i;
    for i in 0..(octets.len() / 32) {
        let self_vec = _mm256_loadu_si256(self_avx_ptr.add(i));
        let other_vec = _mm256_loadu_si256(other_avx_ptr.add(i));
        let result = _mm256_xor_si256(self_vec, other_vec);
        _mm256_storeu_si256(self_avx_ptr.add(i), result);
    }

    let remainder = octets.len() % 32;
    let self_ptr = octets.as_mut_ptr() as *mut u64;
    let other_ptr = other.as_ptr() as *const u64;
    for i in ((octets.len() - remainder) / 8)..(octets.len() / 8) {
        *self_ptr.add(i) ^= *other_ptr.add(i);
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
    }

    return add_assign_fallback(octets, other);
}

#[target_feature(enable = "avx2")]
unsafe fn count_ones_and_nonzeros_avx2(octets: &[u8]) -> (usize, usize) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let avx_ones = _mm256_set1_epi8(1);
    let avx_zeros = _mm256_set1_epi8(0);
    // Safe because _mm256_loadu_si256 loads from unaligned memory
    #[allow(clippy::cast_ptr_alignment)]
    let avx_ptr = octets.as_ptr() as *const __m256i;

    let mut ones = 0;
    let mut non_zeros = 0;
    for i in 0..(octets.len() / 32) {
        let vec = _mm256_loadu_si256(avx_ptr.add(i));
        let compared_ones = _mm256_cmpeq_epi8(vec, avx_ones);
        ones += _mm256_extract_epi64(compared_ones, 0).count_ones() / 8;
        ones += _mm256_extract_epi64(compared_ones, 1).count_ones() / 8;
        ones += _mm256_extract_epi64(compared_ones, 2).count_ones() / 8;
        ones += _mm256_extract_epi64(compared_ones, 3).count_ones() / 8;

        let compared_zeros = _mm256_cmpeq_epi8(vec, avx_zeros);
        non_zeros += 32;
        non_zeros -= _mm256_extract_epi64(compared_zeros, 0).count_ones() / 8;
        non_zeros -= _mm256_extract_epi64(compared_zeros, 1).count_ones() / 8;
        non_zeros -= _mm256_extract_epi64(compared_zeros, 2).count_ones() / 8;
        non_zeros -= _mm256_extract_epi64(compared_zeros, 3).count_ones() / 8;
    }

    let mut remainder = octets.len() % 32;
    if remainder >= 16 {
        remainder -= 16;
        let avx_ones = _mm_set1_epi8(1);
        let avx_zeros = _mm_set1_epi8(0);
        // Safe because _mm_lddqu_si128 loads from unaligned memory
        #[allow(clippy::cast_ptr_alignment)]
        let avx_ptr = octets.as_ptr().add((octets.len() / 32) * 32) as *const __m128i;

        let vec = _mm_lddqu_si128(avx_ptr);
        let compared_ones = _mm_cmpeq_epi8(vec, avx_ones);
        ones += _mm_extract_epi64(compared_ones, 0).count_ones() / 8;
        ones += _mm_extract_epi64(compared_ones, 1).count_ones() / 8;

        let compared_zeros = _mm_cmpeq_epi8(vec, avx_zeros);
        non_zeros += 16;
        non_zeros -= _mm_extract_epi64(compared_zeros, 0).count_ones() / 8;
        non_zeros -= _mm_extract_epi64(compared_zeros, 1).count_ones() / 8;
    }

    for i in (octets.len() - remainder)..octets.len() {
        let value = octets.get_unchecked(i);
        if *value == 1 {
            ones += 1;
        }
        if *value != 0 {
            non_zeros += 1;
        }
    }
    (ones as usize, non_zeros as usize)
}

fn count_ones_and_nonzeros_fallback(octets: &[u8]) -> (usize, usize) {
    let mut ones = 0;
    let mut non_zeros = 0;
    for value in octets.iter() {
        if *value == 1 {
            ones += 1;
        }
        if *value != 0 {
            non_zeros += 1;
        }
    }
    (ones, non_zeros)
}

pub fn count_ones_and_nonzeros(octets: &[u8]) -> (usize, usize) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return count_ones_and_nonzeros_avx2(octets);
            }
        }
    }

    return count_ones_and_nonzeros_fallback(octets);
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
