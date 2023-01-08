use std::ops::{BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign};
use smallvec::{SmallVec, smallvec};

const BITS: usize = usize::BITS as usize;
// Making this larger doubles the runtime of merge_all_constraints
const SMALLVEC_ELEMENTS: usize = 64 / BITS;

/// A simple, partial implementation of a BitVec, implemented due to the complexity
/// that the BitVec library introduces by having its vectors be bit-aligned, which is unnessesary
/// for this project
#[derive(Clone)]
pub struct BitVec {
    /// the set of bits stored in the vector
    /// note that bits outside of the vector are allowed to have arbitrary values
    bits: SmallVec<[usize; SMALLVEC_ELEMENTS]>,
    len: usize
}

impl BitVec {
    pub fn empty() -> Self {
        Self {
            bits: SmallVec::new(),
            len: 0,
        }
    }

    pub fn new(value: bool, len: usize) -> Self {
        let mut bits = smallvec![if value {usize::MAX} else {0}; len / BITS + 1];

        if value {
            if let Some(last) = bits.last_mut() {
                *last = (1 << (len % BITS)) - 1;
            }
        }

        Self { bits, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn resize(&mut self, len: usize, value: bool) {
        use std::cmp::Ordering;
        let new_vec_len = len / BITS + 1;

        match len.cmp(&self.len) {
            Ordering::Less => {
                self.bits.truncate(new_vec_len);
                self.len = len;
            }
            Ordering::Greater => {
                if let Some(last) = self.bits.last_mut() {
                    *last |= if value { !((1 << (len % BITS)) - 1) } else {0};
                }
                self.bits.resize(new_vec_len, if value {usize::MAX} else {0});
                self.len = len;
            }
            Ordering::Equal => {}
        }
    }

    pub fn set(&mut self, idx: usize, value: bool) {
        assert!(idx < self.len);

        let bit = 1 << (idx % BITS);

        self.bits[idx / BITS] &= !bit;
        self.bits[idx / BITS] |= if value {bit} else {0};
    }

    pub fn get(&self, idx: usize) -> Option<bool> {
        if idx >= self.len() {
            None
        } else {
            Some(self.bits[idx / BITS] & (1 << (idx % BITS)) != 0)
        }
    }

    pub fn push(&mut self, value: bool) {
        if self.len % BITS == BITS - 1 {
            self.bits.push(0);
        }

        self.len += 1;
        self.set(self.len - 1, value);
    }

    pub fn leading_zeros(&self) -> usize {
        let Some(i) = self
            .iter_elems()
            .position(|x| x != 0)
            else {return self.len()};

        (i * BITS + self.bits[i].trailing_zeros() as usize).min(self.len())
    }

    pub fn count_ones(&self) -> usize {
        let mut out = self.bits[..self.bits.len().saturating_sub(1)]
            .iter()
            .map(|x| x.count_ones() as usize)
            .sum::<usize>();

        if let Some(last) = self.bits.last() {
            // println!("{} {} {:x}", self.bits.len(), self.len, ((1 << (self.len % BITS)) - 1));
            out += (last & ((1 << (self.len % BITS)) - 1)).count_ones() as usize;
        }

        out
    }

    pub fn has_ones(&self) -> bool {
        if self.bits[..self.bits.len().saturating_sub(1)].iter().any(|x| *x != 0) {
            true
        } else if let Some(last) = self.bits.last() {
            (last & ((1 << (self.len % BITS)) - 1)) != 0
        } else {
            false
        }
    }

    pub fn iter_elems(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.iter().copied()
    }

    pub fn iter_elems_mut(&mut self) -> impl Iterator<Item = &mut usize> {
        self.bits.iter_mut()
    }

    pub fn iter_ones(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter_elems()
            .enumerate()
            .flat_map(|(i, elem)| LocIter(elem).map(move |x| x + i * BITS))
            .filter(|i| *i < self.len())
    }

    pub fn iter(&self) -> BitVecIter {
        BitVecIter {
            vec: self,
            idx: 0,
        }
    }

    pub fn func_assign<F>(&mut self, mut func: F)
        where F: FnMut(usize) -> usize
    {
        for elem in self.iter_elems_mut() {
            *elem = func(*elem);
        }
    }

    pub fn func<F>(&self, func: F) -> BitVec
        where F: FnMut(usize) -> usize
    {
        let mut out = self.clone();
        out.func_assign(func);
        out
    }

    pub fn invert_inplace(&mut self) {
        self.func_assign(|x| !x);
    }

    pub fn op<F>(&self, other: &Self, mut op: F) -> Self
        where F: FnMut(usize, usize) -> usize
    {
        assert_eq!(self.len(), other.len());

        let bits = self.iter_elems()
            .zip(other.iter_elems())
            .map(|(a, b)| op(a, b))
            .collect::<SmallVec<_>>();

        Self {
            bits,
            len: self.len()
        }
    }

    pub fn op_assign<F>(&mut self, other: &Self, mut op: F)
        where F: FnMut(&mut usize, usize)
    {
        assert_eq!(self.len(), other.len());

        for (a, b) in Iterator::zip(self.iter_elems_mut(), other.iter_elems()) {
            op(a, b);
        }
    }

    pub fn or (&self, other: &Self) -> Self { self.op(other, |a, b| a | b) }
    pub fn and(&self, other: &Self) -> Self { self.op(other, |a, b| a & b) }
    pub fn xor(&self, other: &Self) -> Self { self.op(other, |a, b| a ^ b) }

    pub fn or_assign (&mut self, other: &Self) { self.op_assign(other, |a, b| *a |= b) }
    pub fn and_assign(&mut self, other: &Self) { self.op_assign(other, |a, b| *a &= b) }
    pub fn xor_assign(&mut self, other: &Self) { self.op_assign(other, |a, b| *a ^= b) }
}

#[derive(Clone, Copy, Debug)]
pub struct LocIter(pub usize);

impl Iterator for LocIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let out = self.0.trailing_zeros();
            self.0 &= self.0 - 1;
            Some(out as usize)
        }
    }
}

impl BitAnd for &BitVec {
    type Output = BitVec;

    fn bitand(self, other: Self) -> BitVec {
        self.and(other)
    }
}

impl BitOr for &BitVec {
    type Output = BitVec;

    fn bitor(self, other: Self) -> BitVec {
        self.or(other)
    }
}

impl BitXor for &BitVec {
    type Output = BitVec;

    fn bitxor(self, other: Self) -> BitVec {
        self.xor(other)
    }
}

impl BitAndAssign<&BitVec> for BitVec {
    fn bitand_assign(&mut self, other: &Self) {
        self.and_assign(other)
    }
}

impl BitOrAssign<&BitVec> for BitVec {
    fn bitor_assign(&mut self, other: &Self) {
        self.or_assign(other)
    }
}

impl BitXorAssign<&BitVec> for BitVec {
    fn bitxor_assign(&mut self, other: &Self) {
        self.xor_assign(other)
    }
}

impl BitVec {
    // deposits bits from bits into a new bitvec with the same length as self, using self as
    // a mask
    pub fn deposit_bits(&self, mut bits: u128) -> Self {
        let mut out = Self::new(false, self.len());

        for (b, out_b) in self.iter_elems().zip(out.iter_elems_mut()) {
            for i in LocIter(b) {
                *out_b |= (bits as usize & 1) << i;
                bits >>= 1;
            }
        }

        out
    }
}

pub struct BitVecIter<'a> {
    vec: &'a BitVec,
    idx: usize,
}

impl Iterator for BitVecIter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if let Some(out) = self.vec.get(self.idx) {
            self.idx += 1;
            Some(out)
        } else {
            None
        }
    }
}

use std::fmt;

impl fmt::Debug for BitVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();

        for b in self.iter() {
            out += &format!("{}", b as u8);
        }

        write!(f, "{}", &out)
    }
}
