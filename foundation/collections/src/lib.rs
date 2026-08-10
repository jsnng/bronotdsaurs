#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
#[cfg(kani)]
extern crate kani;

use alloc::vec::Vec;
use alloc::vec;

#[derive(Debug, Clone)]
pub enum SmallBytes<const N: usize> {
    Stack { elements: [u8; N], length: usize },
    Heap(Vec<u8>)
}

impl<const N: usize> Default for SmallBytes<N> {
    fn default() -> Self {
        Self::Stack { elements: [0u8; N], length: 0 }
    }
}

impl<const N: usize> SmallBytes<N> {
    /// Build a buffer of `count` bytes by calling `f(i)` for `i` in `0..count`.
    ///
    /// This is the only construction path that produces a non-empty, non-zeroed
    /// buffer from computed values. The buffer is read-only to the caller
    /// afterward.
    pub fn fill_with<F: FnMut(usize) -> u8>(count: usize, mut f: F) -> Self {
        if count <= N {
            let mut elements = [0u8; N];
            for (i, el) in elements.iter_mut().enumerate().take(count) {
                *el = f(i);
            }
            Self::Stack { elements, length: count }
        } else {
            let mut v = Vec::with_capacity(count);
            for i in 0..count {
                v.push(f(i));
            }
            Self::Heap(v)
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Stack { length, .. } => *length,
            Self::Heap(vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<const N: usize> SmallBytes<N> {
    pub fn from_slice(slice: &[u8]) -> Self {
        if slice.len() <= N {
            let mut elements = [0u8; N];
            elements[..slice.len()].copy_from_slice(slice);
            Self::Stack { elements, length: slice.len() }
        } else {
            Self::Heap(slice.to_vec())
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Stack { elements, length } => &elements[..*length],
            Self::Heap(vec) => vec.as_slice(),
        }
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
}

impl<const N: usize> core::ops::Index<usize> for SmallBytes<N> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<const N: usize> SmallBytes<N> {
    pub fn with_len(len: usize) -> Self {
        if len <= N {
            Self::Stack {
                elements: [0u8; N],
                length: len,
            }
        } else {
            Self::Heap(vec![0u8; len])
        }
    }

    pub fn get(&self, index: usize) -> Option<u8> {
        match self {
            Self::Stack { elements, length } => {
                if index < *length {
                    Some(elements[index])
                } else {
                    None
                }
            }
            Self::Heap(elements) => elements.get(index).copied(),
        }
    }

}