use super::{BitPacker, UnsafeBitPacker};

#[cfg(target_arch = "x86_64")]
use crate::Available;

const BLOCK_LEN: usize = 32 * 4;

#[cfg(any(target_arch = "aarch64"))]
mod neon {

    use super::BLOCK_LEN;
    use crate::Available;

    use std::arch::aarch64::uint32x4_t as DataType;
    use std::arch::aarch64::vaddq_u32;
    use std::arch::aarch64::vandq_u32;
    use std::arch::aarch64::vdupq_n_s32;
    use std::arch::aarch64::vdupq_n_u32;
    use std::arch::aarch64::vld1q_u32;
    use std::arch::aarch64::vorrq_u32;
    use std::arch::aarch64::vshlq_u32;
    use std::arch::aarch64::vshrq_u32;
    use std::arch::aarch64::vst1q_u32;
    use std::arch::aarch64::vsubq_u32;

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn left_shift_32(el: DataType, shift: i32) -> DataType {
        vshlq_u32(a, vdupq_n_s32(imm))
    }

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn right_shift_32(el: DataType, shift: i32) -> DataType {
        vshrq_u32(a, vdupq_n_u32(imm))
    }

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn set1(el: i32) -> DataType {
        vdupq_n_u32(el as u32)
    }

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn op_and(a: DataType, b: DataType) -> DataType {
        vandq_u32(a, b)
    }

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn op_or(a: DataType, b: DataType) -> DataType {
        vorrq_u32(a, b)
    }

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn load_unaligned(p: *const DataType) -> DataType {
        vld1q_u32(p)
    }

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn store_unaligned(addr: *mut DataType, data: DataType) {
        vst1q_u32(addr as (*mut u32), data)
    }

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn or_collapse_to_u32(accumulator: DataType) -> u32 {
        let a__b__c__d_ = accumulator;
        let ______a__b_ = vshrq_n_u32(a__b__c__d_, 8);
        let a__b__ca_db = op_or(a__b__c__d_, ______a__b_);
        let ___a__b__ca = vshrq_n_u32(a__b__ca_db, 4);
        let _______cadb = op_or(a__b__ca_db, ___a__b__ca);
        vgetq_lane_u32(_______cadb, 0)
    }

    #[target_feature(enable = "neon")]
    unsafe fn compute_delta(curr: DataType, prev: DataType) -> DataType {
        vsubq_u32(curr, op_or(vshlq_n_u32(curr, 4), vshlq_n_u32(curr, 12)))
    }

    #[target_feature(enable = "neon")]
    #[allow(non_snake_case)]
    #[inline]
    unsafe fn integrate_delta(prev: DataType, delta: DataType) -> DataType {
        let offset = vdupq_n_s32(prev, uint32x4_t[3]);
        let a__b__c__d_ = delta;
        let ______a__b_ = vshlq_n_u32(delta, 8);
        let a__b__ca_db = vaddq_u32(______a__b_, a__b__c__d_);
        let ___a__b__ca = vshlq_n_u32(a__b__ca_db, 4);
        let a_ab_abc_abcd: DataType = vaddq_u32(___a__b__ca, a__b__ca_db);
        vaddq_u32(offset, a_ab_abc_abcd)
    }

    declare_bitpacker!(target_feature(enable = "neon"));

    impl Available for UnsafeBitPackerImpl {
        fn available() -> bool {
            is_aarch64_feature_detected!("neon")
        }
    }
}

#[cfg(any(target_arch = "x86_64"))]
mod sse3 {

    use super::BLOCK_LEN;
    use crate::Available;

    use std::arch::x86_64::__m128i as DataType;
    use std::arch::x86_64::_mm_and_si128 as op_and;
    use std::arch::x86_64::_mm_lddqu_si128 as load_unaligned;
    use std::arch::x86_64::_mm_or_si128 as op_or;
    use std::arch::x86_64::_mm_set1_epi32 as set1;
    use std::arch::x86_64::_mm_slli_epi32 as left_shift_32;
    use std::arch::x86_64::_mm_srli_epi32 as right_shift_32;
    use std::arch::x86_64::_mm_storeu_si128 as store_unaligned;
    use std::arch::x86_64::{
        _mm_add_epi32, _mm_cvtsi128_si32, _mm_shuffle_epi32, _mm_slli_si128, _mm_srli_si128,
        _mm_sub_epi32,
    };

    #[allow(non_snake_case)]
    #[inline]
    unsafe fn or_collapse_to_u32(accumulator: DataType) -> u32 {
        let a__b__c__d_ = accumulator;
        let ______a__b_ = _mm_srli_si128(a__b__c__d_, 8);
        let a__b__ca_db = op_or(a__b__c__d_, ______a__b_);
        let ___a__b__ca = _mm_srli_si128(a__b__ca_db, 4);
        let _______cadb = op_or(a__b__ca_db, ___a__b__ca);
        _mm_cvtsi128_si32(_______cadb) as u32
    }

    #[target_feature(enable = "sse3")]
    unsafe fn compute_delta(curr: DataType, prev: DataType) -> DataType {
        _mm_sub_epi32(
            curr,
            op_or(_mm_slli_si128(curr, 4), _mm_srli_si128(prev, 12)),
        )
    }

    #[target_feature(enable = "sse3")]
    #[allow(non_snake_case)]
    #[inline]
    unsafe fn integrate_delta(prev: DataType, delta: DataType) -> DataType {
        let offset = _mm_shuffle_epi32(prev, 0xff);
        let a__b__c__d_ = delta;
        let ______a__b_ = _mm_slli_si128(delta, 8);
        let a__b__ca_db = _mm_add_epi32(______a__b_, a__b__c__d_);
        let ___a__b__ca = _mm_slli_si128(a__b__ca_db, 4);
        let a_ab_abc_abcd: DataType = _mm_add_epi32(___a__b__ca, a__b__ca_db);
        _mm_add_epi32(offset, a_ab_abc_abcd)
    }

    declare_bitpacker!(target_feature(enable = "sse3"));

    impl Available for UnsafeBitPackerImpl {
        fn available() -> bool {
            is_x86_feature_detected!("sse3")
        }
    }
}

mod scalar {

    use super::BLOCK_LEN;
    use crate::Available;
    use std::ptr;

    type DataType = [u32; 4];

    fn set1(el: i32) -> DataType {
        [el as u32; 4]
    }

    fn right_shift_32(el: DataType, shift: i32) -> DataType {
        [
            el[0] >> shift,
            el[1] >> shift,
            el[2] >> shift,
            el[3] >> shift,
        ]
    }

    fn left_shift_32(el: DataType, shift: i32) -> DataType {
        [
            el[0] << shift,
            el[1] << shift,
            el[2] << shift,
            el[3] << shift,
        ]
    }

    fn op_or(left: DataType, right: DataType) -> DataType {
        [
            left[0] | right[0],
            left[1] | right[1],
            left[2] | right[2],
            left[3] | right[3],
        ]
    }

    fn op_and(left: DataType, right: DataType) -> DataType {
        [
            left[0] & right[0],
            left[1] & right[1],
            left[2] & right[2],
            left[3] & right[3],
        ]
    }

    unsafe fn load_unaligned(addr: *const DataType) -> DataType {
        ptr::read_unaligned(addr)
    }

    unsafe fn store_unaligned(addr: *mut DataType, data: DataType) {
        ptr::write_unaligned(addr, data);
    }

    fn or_collapse_to_u32(accumulator: DataType) -> u32 {
        (accumulator[0] | accumulator[1]) | (accumulator[2] | accumulator[3])
    }

    fn compute_delta(curr: DataType, prev: DataType) -> DataType {
        [
            curr[0].wrapping_sub(prev[3]),
            curr[1].wrapping_sub(curr[0]),
            curr[2].wrapping_sub(curr[1]),
            curr[3].wrapping_sub(curr[2]),
        ]
    }

    fn integrate_delta(offset: DataType, delta: DataType) -> DataType {
        let el0 = offset[3].wrapping_add(delta[0]);
        let el1 = el0.wrapping_add(delta[1]);
        let el2 = el1.wrapping_add(delta[2]);
        let el3 = el2.wrapping_add(delta[3]);
        [el0, el1, el2, el3]
    }

    // The `cfg(any(debug, not(debug)))` is here to put an attribute that has no effect.
    //
    // For other bitpacker, we enable specific CPU instruction set, but for the
    // scalar bitpacker none is required.
    declare_bitpacker!(cfg(any(debug, not(debug))));

    impl Available for UnsafeBitPackerImpl {
        fn available() -> bool {
            true
        }
    }
}

#[derive(Clone, Copy)]
enum InstructionSet {
    #[cfg(target_arch = "x86_64")]
    SSE3,
    Scalar,
}

/// `BitPacker4x` packs integers in groups of 4. This gives an opportunity
/// to leverage `SSE3` instructions to encode and decode the stream.
///
/// One block must contain `128 integers`.
#[derive(Clone, Copy)]
pub struct BitPacker4x(InstructionSet);

impl BitPacker for BitPacker4x {
    const BLOCK_LEN: usize = BLOCK_LEN;

    /// Returns the best available implementation for the current CPU.
    fn new() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if sse3::UnsafeBitPackerImpl::available() {
                return BitPacker4x(InstructionSet::SSE3);
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if neon::UnsafeBitPackerImpl::available() {
                return BitPacker4x(InstructionSet::Neon);
            }
        }
        BitPacker4x(InstructionSet::Scalar)
    }

    fn compress(&self, decompressed: &[u32], compressed: &mut [u8], num_bits: u8) -> usize {
        unsafe {
            match self.0 {
                #[cfg(target_arch = "x86_64")]
                InstructionSet::SSE3 => {
                    sse3::UnsafeBitPackerImpl::compress(decompressed, compressed, num_bits)
                }
                #[cfg(target_arch = "aarch64")]
                InstructionSet::Neon => {
                    neon::UnsafeBitPackerImpl::compress(decompressed, compressed, num_bits)
                }
                InstructionSet::Scalar => {
                    scalar::UnsafeBitPackerImpl::compress(decompressed, compressed, num_bits)
                }
            }
        }
    }

    fn compress_sorted(
        &self,
        initial: u32,
        decompressed: &[u32],
        compressed: &mut [u8],
        num_bits: u8,
    ) -> usize {
        unsafe {
            match self.0 {
                #[cfg(target_arch = "x86_64")]
                InstructionSet::SSE3 => sse3::UnsafeBitPackerImpl::compress_sorted(
                    initial,
                    decompressed,
                    compressed,
                    num_bits,
                ),
                #[cfg(target_arch = "aarch64")]
                InstructionSet::Neon => neon::UnsafeBitPackerImpl::compress_sorted(
                    initial,
                    decompressed,
                    compressed,
                    num_bits,
                ),
                InstructionSet::Scalar => scalar::UnsafeBitPackerImpl::compress_sorted(
                    initial,
                    decompressed,
                    compressed,
                    num_bits,
                ),
            }
        }
    }

    fn decompress(&self, compressed: &[u8], decompressed: &mut [u32], num_bits: u8) -> usize {
        unsafe {
            match self.0 {
                #[cfg(target_arch = "x86_64")]
                InstructionSet::SSE3 => {
                    sse3::UnsafeBitPackerImpl::decompress(compressed, decompressed, num_bits)
                }
                #[cfg(target_arch = "aarch64")]
                InstructionSet::Neon => {
                    neon::UnsafeBitPackerImpl::decompress(compressed, decompressed, num_bits)
                }
                InstructionSet::Scalar => {
                    scalar::UnsafeBitPackerImpl::decompress(compressed, decompressed, num_bits)
                }
            }
        }
    }

    fn decompress_sorted(
        &self,
        initial: u32,
        compressed: &[u8],
        decompressed: &mut [u32],
        num_bits: u8,
    ) -> usize {
        unsafe {
            match self.0 {
                #[cfg(target_arch = "x86_64")]
                InstructionSet::SSE3 => sse3::UnsafeBitPackerImpl::decompress_sorted(
                    initial,
                    compressed,
                    decompressed,
                    num_bits,
                ),
                #[cfg(target_arch = "aarch64")]
                InstructionSet::Neon => neon::UnsafeBitPackerImpl::decompress_sorted(
                    initial,
                    compressed,
                    decompressed,
                    num_bits,
                ),
                InstructionSet::Scalar => scalar::UnsafeBitPackerImpl::decompress_sorted(
                    initial,
                    compressed,
                    decompressed,
                    num_bits,
                ),
            }
        }
    }

    fn num_bits(&self, decompressed: &[u32]) -> u8 {
        unsafe {
            match self.0 {
                #[cfg(target_arch = "x86_64")]
                InstructionSet::SSE3 => sse3::UnsafeBitPackerImpl::num_bits(decompressed),
                #[cfg(target_arch = "aarch64")]
                InstructionSet::Neon => neon::UnsafeBitPackerImpl::num_bits(decompressed),
                InstructionSet::Scalar => scalar::UnsafeBitPackerImpl::num_bits(decompressed),
            }
        }
    }

    fn num_bits_sorted(&self, initial: u32, decompressed: &[u32]) -> u8 {
        unsafe {
            match self.0 {
                #[cfg(target_arch = "x86_64")]
                InstructionSet::SSE3 => {
                    sse3::UnsafeBitPackerImpl::num_bits_sorted(initial, decompressed)
                }
                #[cfg(target_arch = "aarch64")]
                InstructionSet::Neon => {
                    neon::UnsafeBitPackerImpl::num_bits_sorted(initial, decompressed)
                }
                InstructionSet::Scalar => {
                    scalar::UnsafeBitPackerImpl::num_bits_sorted(initial, decompressed)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BLOCK_LEN;
    use super::{scalar, sse3};
    use crate::tests::test_util_compatible;
    use crate::Available;
    use crate::{BitPacker, BitPacker4x};

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_compatible_sse3() {
        if sse3::UnsafeBitPackerImpl::available() {
            test_util_compatible::<scalar::UnsafeBitPackerImpl, sse3::UnsafeBitPackerImpl>(
                BLOCK_LEN,
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_compatible_neon() {
        if neon::UnsafeBitPackerImpl::available() {
            test_util_compatible::<scalar::UnsafeBitPackerImpl, neon::UnsafeBitPackerImpl>(
                BLOCK_LEN,
            );
        }
    }

    #[test]
    fn test_delta_bit_width_32() {
        let values = vec![i32::max_value() as u32 + 1; BitPacker4x::BLOCK_LEN];
        let bit_packer = BitPacker4x::new();
        let bit_width = bit_packer.num_bits_sorted(0, &values);
        assert_eq!(bit_width, 32);

        let mut block = vec![0u8; BitPacker4x::compressed_block_size(bit_width)];
        bit_packer.compress_sorted(0, &values, &mut block, bit_width);

        let mut decoded_values = vec![0x10101010; BitPacker4x::BLOCK_LEN];
        bit_packer.decompress_sorted(0, &block, &mut decoded_values, bit_width);

        assert_eq!(values, decoded_values);
    }

    #[test]
    fn test_bit_width_32() {
        let mut values = vec![i32::max_value() as u32 + 1; BitPacker4x::BLOCK_LEN];
        values[0] = 0;
        let bit_packer = BitPacker4x::new();
        let bit_width = bit_packer.num_bits(&values);
        assert_eq!(bit_width, 32);

        let mut block = vec![0u8; BitPacker4x::compressed_block_size(bit_width)];
        bit_packer.compress(&values, &mut block, bit_width);

        let mut decoded_values = vec![0x10101010; BitPacker4x::BLOCK_LEN];
        bit_packer.decompress(&block, &mut decoded_values, bit_width);

        assert_eq!(values, decoded_values);
    }
}
