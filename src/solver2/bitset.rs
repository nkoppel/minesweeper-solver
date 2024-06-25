use smallvec::{smallvec, SmallVec};
use std::ops::*;
use std::simd::{cmp::SimdPartialEq, mask64x8, num::SimdUint, u64x8};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitSet {
    bits: SmallVec<[u64x8; 2]>,
    len: usize,
}

impl BitSet {
    pub fn zeros(len: usize) -> Self {
        let num_vecs = len.div_ceil(512);

        Self {
            bits: smallvec![u64x8::splat(0); num_vecs],
            len,
        }
    }

    fn clear_extra_bits(&mut self) {
        let len = self.len;
        let slice = self.slice_view_mut();

        slice[len.div_ceil(64)..].fill(0);
        slice[len / 64] = (1u64 << (len % 64)).wrapping_sub(1);
    }

    pub fn ones(len: usize) -> Self {
        let num_vecs = len.div_ceil(512);

        let mut out = Self {
            bits: smallvec![u64x8::splat(u64::MAX); num_vecs],
            len,
        };
        out.clear_extra_bits();

        out
    }

    pub fn slice_view(&self) -> &[u64] {
        unsafe { std::slice::from_raw_parts(self.bits.as_ptr() as *const u64, self.bits.len() * 8) }
    }

    pub fn slice_view_mut(&mut self) -> &mut [u64] {
        unsafe {
            std::slice::from_raw_parts_mut(self.bits.as_mut_ptr() as *mut u64, self.bits.len() * 8)
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(clippy::implied_bounds_in_impls)]
    pub fn iter_ones(&self) -> impl Iterator<Item = usize> + DoubleEndedIterator + Clone + '_ {
        self.slice_view()
            .iter()
            .enumerate()
            .flat_map(|(i, &x)| IterOnes(x).map(move |j| i * 64 + j))
    }

    #[allow(clippy::implied_bounds_in_impls)]
    pub fn into_iter_ones(self) -> impl Iterator<Item = usize> + Clone {
        self.bits
            .into_iter()
            .flat_map(u64x8::to_array)
            .enumerate()
            .flat_map(|(i, x)| IterOnes(x).map(move |j| i * 64 + j))
    }

    fn combine(&self, other: &Self, f: impl Fn(u64x8, u64x8) -> u64x8) -> Self {
        assert_eq!(self.len, other.len);

        let bits = self
            .bits
            .iter()
            .zip(&other.bits)
            .map(|(a, b)| f(*a, *b))
            .collect();

        Self {
            bits,
            len: self.len,
        }
    }

    fn combine_assign(&mut self, other: &Self, f: impl Fn(&mut u64x8, u64x8)) {
        assert_eq!(self.len, other.len);

        for (a, b) in self.bits.iter_mut().zip(&other.bits) {
            f(a, *b)
        }
    }

    pub fn any(&self) -> bool {
        self.bits.iter().any(|&vec| vec != u64x8::splat(0))
    }

    pub fn all(&self) -> bool {
        if !self.bits[..self.bits.len() - 1]
            .iter()
            .all(|&vec| vec == u64x8::splat(u64::MAX))
        {
            return false;
        }

        let Some(last) = self.bits.last() else {
            return true;
        };

        let idx = self.len % 512;

        if !last[..idx / 64].iter().all(|&x| x == u64::MAX) {
            return false;
        }

        last[idx / 64] == (1u64 << (idx % 64)).wrapping_sub(1)
    }

    pub fn equal_on_mask(&self, other: &Self, mask: &Self) -> bool {
        assert_eq!(self.len, other.len);
        assert_eq!(self.len, mask.len);

        self.bits
            .iter()
            .zip(&other.bits)
            .zip(&mask.bits)
            .map(|((a, b), m)| (a & m).simd_eq(b & m))
            .fold(mask64x8::splat(true), |fold, val| fold & val)
            .all()
    }

    pub fn set_to_one(&mut self, idx: usize) {
        assert!(idx < self.len);

        self.slice_view_mut()[idx / 64] |= 1 << (idx % 64);
    }

    pub fn set_to_zero(&mut self, idx: usize) {
        assert!(idx < self.len);

        self.slice_view_mut()[idx / 64] &= !(1 << (idx % 64));
    }

    pub fn set_to(&mut self, idx: usize, value: bool) {
        if value {
            self.set_to_one(idx)
        } else {
            self.set_to_zero(idx)
        }
    }

    pub fn get(&self, idx: usize) -> bool {
        (self.slice_view()[idx / 64] >> (idx % 64)) & 1 == 1
    }

    pub fn invert(&mut self) {
        for x in &mut self.bits {
            *x = !*x;
        }

        self.clear_extra_bits();
    }

    pub fn inverted(&self) -> Self {
        let mut out = self.clone();
        out.invert();
        out
    }

    pub fn overlaps_with(&self, other: &Self) -> bool {
        self.bits
            .iter()
            .zip(&other.bits)
            .any(|(a, b)| a & b != u64x8::splat(0))
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.bits.iter().zip(&other.bits).all(|(a, b)| a & b == *a)
    }
}

// Should hopefully compile down into intrinsics
fn count_vec_ones(vec: u64x8) -> u64x8 {
    u64x8::from_array(vec.to_array().map(|x| x.count_ones() as u64))
}

impl BitSet {
    pub fn count_ones(&self) -> usize {
        self.bits
            .iter()
            .copied()
            .map(count_vec_ones)
            .sum::<u64x8>()
            .reduce_sum() as usize
    }

    pub fn count_overlap_ones(&self, other: &BitSet) -> usize {
        assert_eq!(self.len, other.len);

        self.bits
            .iter()
            .zip(&other.bits)
            .map(|(a, b)| count_vec_ones(a & b))
            .sum::<u64x8>()
            .reduce_sum() as usize
    }
}

macro_rules! impl_combination_operator {
    ($op:ident, $fn:ident, $func:expr) => {
        impl $op for BitSet {
            type Output = BitSet;

            fn $fn(self, rhs: BitSet) -> BitSet {
                self.combine(&rhs, $func)
            }
        }

        impl $op for &BitSet {
            type Output = BitSet;

            fn $fn(self, rhs: &BitSet) -> BitSet {
                self.combine(rhs, $func)
            }
        }

        impl $op<&BitSet> for BitSet {
            type Output = BitSet;

            fn $fn(self, rhs: &BitSet) -> BitSet {
                self.combine(rhs, $func)
            }
        }

        impl $op<BitSet> for &BitSet {
            type Output = BitSet;

            fn $fn(self, rhs: BitSet) -> BitSet {
                self.combine(&rhs, $func)
            }
        }
    };
}

macro_rules! impl_assignment_operator {
    ($op:ident, $fn:ident, $func:expr) => {
        impl $op for BitSet {
            fn $fn(&mut self, rhs: BitSet) {
                self.combine_assign(&rhs, $func)
            }
        }

        impl $op<&BitSet> for BitSet {
            fn $fn(&mut self, rhs: &BitSet) {
                self.combine_assign(rhs, $func)
            }
        }
    };
}

impl_combination_operator!(BitAnd, bitand, |a, b| a & b);
impl_combination_operator!(BitOr, bitor, |a, b| a | b);
impl_combination_operator!(BitXor, bitxor, |a, b| a ^ b);
impl_combination_operator!(Add, add, |a, b| a | b);
impl_combination_operator!(Sub, sub, |a, b| a & !b);

impl_assignment_operator!(BitAndAssign, bitand_assign, |a, b| *a &= b);
impl_assignment_operator!(BitOrAssign, bitor_assign, |a, b| *a |= b);
impl_assignment_operator!(BitXorAssign, bitxor_assign, |a, b| *a ^= b);
impl_assignment_operator!(AddAssign, add_assign, |a, b| *a |= b);
impl_assignment_operator!(SubAssign, sub_assign, |a, b| *a &= !b);

impl Not for BitSet {
    type Output = BitSet;

    fn not(self) -> Self::Output {
        self.inverted()
    }
}

impl Not for &BitSet {
    type Output = BitSet;

    fn not(self) -> Self::Output {
        self.inverted()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct IterOnes(pub u64);

impl Iterator for IterOnes {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let out = self.0.trailing_zeros();
        self.0 ^= 1 << out;
        Some(out as usize)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.0.count_ones() as usize;
        (size, Some(size))
    }
}

impl DoubleEndedIterator for IterOnes {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let out = 63 - self.0.leading_zeros();
        self.0 ^= 1 << out;
        Some(out as usize)
    }
}

impl std::fmt::Debug for BitSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();

        for x in self.slice_view() {
            for i in 0..64 {
                if x & (1 << i) != 0 {
                    out.push('1');
                } else {
                    out.push('0');
                }
            }
        }

        out.truncate(self.len());
        write!(f, "{out}")
    }
}
