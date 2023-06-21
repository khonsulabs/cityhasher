#![doc = include_str!("../README.md")]
#![no_std]
#![warn(missing_docs, clippy::all)]
#![cfg_attr(feature = "disable-bounds-checking", deny(unsafe_code))]
#![cfg_attr(not(feature = "disable-bounds-checking"), forbid(unsafe_code))]

use core::hash::{BuildHasher, Hasher};
use core::mem;

#[cfg(feature = "std")]
extern crate std;

struct Input<'a>(&'a [u8]);

impl<'a> Input<'a> {
    const fn len(&self) -> usize {
        self.0.len()
    }

    #[cfg_attr(feature = "disable-bounds-checking", allow(unsafe_code))]
    fn fetch32(&self, offset: usize) -> u32 {
        u32::from_le_bytes({
            #[cfg(feature = "disable-bounds-checking")]
            unsafe {
                self.0
                    .get_unchecked(offset..offset + 4)
                    .try_into()
                    .unwrap_unchecked()
            }
            #[cfg(not(feature = "disable-bounds-checking"))]
            self.0[offset..offset + 4]
                .try_into()
                .expect("u32 is 4 bytes")
        })
    }

    #[cfg_attr(feature = "disable-bounds-checking", allow(unsafe_code))]
    fn fetch64(&self, offset: usize) -> u64 {
        u64::from_le_bytes({
            #[cfg(feature = "disable-bounds-checking")]
            unsafe {
                self.0
                    .get_unchecked(offset..offset + 8)
                    .try_into()
                    .unwrap_unchecked()
            }
            #[cfg(not(feature = "disable-bounds-checking"))]
            self.0[offset..offset + 8]
                .try_into()
                .expect("u64 is 8 bytes")
        })
    }

    fn hash32_len_13_to_24(&self) -> u32 {
        #[cfg(not(feature = "disable-bounds-checking"))]
        assert!(self.len() <= 24); // This helps the optimizer with bounds checking
        let a = self.fetch32((self.len() >> 1) - 4);
        let b = self.fetch32(4);
        let c = self.fetch32(self.len() - 8);
        let d = self.fetch32(self.len() >> 1);
        let e = self.fetch32(0);
        let f = self.fetch32(self.len() - 4);
        let h = self.len() as u32;

        fmix(mur(f, mur(e, mur(d, mur(c, mur(b, mur(a, h)))))))
    }

    fn hash32_len_0_to_4(&self) -> u32 {
        assert!(self.len() <= 4); // This helps the optimizer with automatic loop unrolling.
        let mut b = 0_u32;
        let mut c = 9;
        for v in self.0 {
            let as_signed = *v as i8;

            b = b.wrapping_mul(C1).wrapping_add(as_signed as u32);
            c ^= b;
        }
        fmix(mur(b, mur(self.len() as u32, c)))
    }

    fn hash32_len_5_to_12(&self) -> u32 {
        #[cfg(not(feature = "disable-bounds-checking"))]
        assert!(self.len() <= 12);
        let mut a = self.len() as u32;
        let mut b = a.wrapping_mul(5);
        let mut c = 9_u32;
        let d = b;

        a = a.wrapping_add(self.fetch32(0));
        b = b.wrapping_add(self.fetch32(self.len() - 4));
        c = c.wrapping_add(self.fetch32((self.len() >> 1) & 4));
        fmix(mur(c, mur(b, mur(a, d))))
    }

    fn hash32(&self) -> u32 {
        if self.len() <= 24 {
            return if self.len() <= 12 {
                if self.len() <= 4 {
                    self.hash32_len_0_to_4()
                } else {
                    self.hash32_len_5_to_12()
                }
            } else {
                self.hash32_len_13_to_24()
            };
        }

        // len > 24
        let mut h = self.len() as u32;
        let mut g = h.wrapping_mul(C1);
        let mut f = g;
        let a0 = rotate32(self.fetch32(self.len() - 4).wrapping_mul(C1), 17).wrapping_mul(C2);
        let a1 = rotate32(self.fetch32(self.len() - 8).wrapping_mul(C1), 17).wrapping_mul(C2);
        let a2 = rotate32(self.fetch32(self.len() - 16).wrapping_mul(C1), 17).wrapping_mul(C2);
        let a3 = rotate32(self.fetch32(self.len() - 12).wrapping_mul(C1), 17).wrapping_mul(C2);
        let a4 = rotate32(self.fetch32(self.len() - 20).wrapping_mul(C1), 17).wrapping_mul(C2);
        h ^= a0;
        h = rotate32(h, 19);
        h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
        h ^= a2;
        h = rotate32(h, 19);
        h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
        g ^= a1;
        g = rotate32(g, 19);
        g = g.wrapping_mul(5).wrapping_add(0xe6546b64);
        g ^= a3;
        g = rotate32(g, 19);
        g = g.wrapping_mul(5).wrapping_add(0xe6546b64);
        f = f.wrapping_add(a4);
        f = rotate32(f, 19);
        f = f.wrapping_mul(5).wrapping_add(0xe6546b64);
        let mut iters = (self.0.len() - 1) / 20;
        let mut offset = 0;

        loop {
            let a0 = rotate32(self.fetch32(offset).wrapping_mul(C1), 17).wrapping_mul(C2);
            let a1 = self.fetch32(offset + 4);
            let a2 = rotate32(self.fetch32(offset + 8).wrapping_mul(C1), 17).wrapping_mul(C2);
            let a3 = rotate32(self.fetch32(offset + 12).wrapping_mul(C1), 17).wrapping_mul(C2);
            let a4 = self.fetch32(offset + 16);
            h ^= a0;
            h = rotate32(h, 18);
            h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
            f = f.wrapping_add(a1);
            f = rotate32(f, 19);
            f = f.wrapping_mul(C1);
            g = g.wrapping_add(a2);
            g = rotate32(g, 18);
            g = g.wrapping_mul(5).wrapping_add(0xe6546b64);
            h ^= a3.wrapping_add(a1);
            h = rotate32(h, 19);
            h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
            g ^= a4;
            g = g.swap_bytes().wrapping_mul(5);
            h = h.wrapping_add(a4.wrapping_mul(5));
            h = h.swap_bytes();
            f = f.wrapping_add(a0);
            permute3(&mut f, &mut h, &mut g);
            offset += 20;
            iters -= 1;
            if iters == 0 {
                break;
            }
        }
        g = rotate32(g, 11).wrapping_mul(C1);
        g = rotate32(g, 17).wrapping_mul(C1);
        f = rotate32(f, 11).wrapping_mul(C1);
        f = rotate32(f, 17).wrapping_mul(C1);
        h = rotate32(h.wrapping_add(g), 19);
        h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
        h = rotate32(h, 17).wrapping_mul(C1);
        h = rotate32(h.wrapping_add(f), 19);
        h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
        h = rotate32(h, 17).wrapping_mul(C1);
        h
    }

    fn hash64(&self) -> u64 {
        if self.len() <= 32 {
            if self.len() <= 16 {
                return self.hash64_len_0_to_16();
            } else {
                return self.hash64_len_17_to_32();
            }
        } else if self.len() <= 64 {
            return self.hash64_len_33_to_64();
        }

        // For strings over 64 bytes we hash the end first, and then as we
        // loop we keep 56 bytes of state: v, w, x, y, and z.
        let mut x = self.fetch64(self.len() - 40);
        let mut y = self
            .fetch64(self.len() - 16)
            .wrapping_add(self.fetch64(self.len() - 56));
        let mut z = hash_len_16_u64(
            self.fetch64(self.len() - 48)
                .wrapping_add(self.len() as u64),
            self.fetch64(self.len() - 24),
        );
        let mut v = self.weak_hash_len_32_with_seeds(self.len() - 64, self.len() as u64, z);
        let mut w = self.weak_hash_len_32_with_seeds(self.len() - 32, y.wrapping_add(K1), x);
        x = x.wrapping_mul(K1).wrapping_add(self.fetch64(0));

        // Decrease len to the nearest multiple of 64, and operate on 64-byte chunks.
        for chunk in self.0[0..self.len() - 1].chunks_exact(64) {
            let chunk = Self(chunk);
            x = rotate64(
                x.wrapping_add(y)
                    .wrapping_add(v.0)
                    .wrapping_add(chunk.fetch64(8)),
                37,
            )
            .wrapping_mul(K1);
            y = rotate64(y.wrapping_add(v.1).wrapping_add(chunk.fetch64(48)), 42).wrapping_mul(K1);
            x ^= w.1;
            y = y.wrapping_add(v.0.wrapping_add(chunk.fetch64(40)));
            z = rotate64(z.wrapping_add(w.0), 33).wrapping_mul(K1);
            v = chunk.weak_hash_len_32_with_seeds(0, v.1.wrapping_mul(K1), x.wrapping_add(w.0));
            w = chunk.weak_hash_len_32_with_seeds(
                32,
                z.wrapping_add(w.1),
                y.wrapping_add(chunk.fetch64(16)),
            );
            mem::swap(&mut z, &mut x);
        }
        hash_len_16_u64(
            hash_len_16_u64(v.0, w.0).wrapping_add(shift_mix(y).wrapping_mul(K1).wrapping_add(z)),
            hash_len_16_u64(v.1, w.1).wrapping_add(x),
        )
    }

    fn hash64_len_0_to_16(&self) -> u64 {
        if self.len() >= 8 {
            let mul = K2.wrapping_add((self.len() as u64).wrapping_mul(2));
            let a = self.fetch64(0).wrapping_add(K2);
            let b = self.fetch64(self.len() - 8);
            let c = rotate64(b, 37).wrapping_mul(mul).wrapping_add(a);
            let d = rotate64(a, 25).wrapping_add(b).wrapping_mul(mul);
            hash_len_16_with_mul(c, d, mul)
        } else if self.len() >= 4 {
            let mul = K2.wrapping_add((self.len() as u64).wrapping_mul(2));
            let a = self.fetch32(0) as u64;
            hash_len_16_with_mul(
                (self.len() as u64).wrapping_add(a << 3),
                self.fetch32(self.len() - 4) as u64,
                mul,
            )
        } else if self.len() > 0 {
            let a = self.0[0] as u32;
            let b = self.0[self.len() >> 1] as u32;
            let c = self.0[self.len() - 1] as u32;
            let y = a.wrapping_add(b << 8);
            let z = (self.len() as u32).wrapping_add(c << 2);
            shift_mix((y as u64).wrapping_mul(K2) ^ (z as u64).wrapping_mul(K0)).wrapping_mul(K2)
        } else {
            K2
        }
    }

    fn hash64_len_17_to_32(&self) -> u64 {
        let mul = K2.wrapping_add(self.len() as u64 * 2);
        let a = self.fetch64(0).wrapping_mul(K1);
        let b = self.fetch64(8);
        let c = self.fetch64(self.len() - 8).wrapping_mul(mul);
        let d = self.fetch64(self.len() - 16).wrapping_mul(K2);
        hash_len_16_with_mul(
            rotate64(a.wrapping_add(b), 43)
                .wrapping_add(rotate64(c, 30))
                .wrapping_add(d),
            a.wrapping_add(rotate64(b.wrapping_add(K2), 18))
                .wrapping_add(c),
            mul,
        )
    }

    fn hash64_len_33_to_64(&self) -> u64 {
        let mul = K2.wrapping_add(self.len() as u64 * 2);
        let a = self.fetch64(0).wrapping_mul(K2);
        let b = self.fetch64(8);
        let c = self.fetch64(self.len() - 24);
        let d = self.fetch64(self.len() - 32);
        let e = self.fetch64(16).wrapping_mul(K2);
        let f = self.fetch64(24).wrapping_mul(9);
        let g = self.fetch64(self.len() - 8);
        let h = self.fetch64(self.len() - 16).wrapping_mul(mul);
        let u = rotate64(a.wrapping_add(g), 43)
            .wrapping_add(rotate64(b, 30).wrapping_add(c).wrapping_mul(9));
        let v = (a.wrapping_add(g) ^ d).wrapping_add(f).wrapping_add(1);
        let w = ((u.wrapping_add(v)).wrapping_mul(mul))
            .swap_bytes()
            .wrapping_add(h);
        let x = rotate64(e.wrapping_add(f), 42).wrapping_add(c);
        let y = (((v.wrapping_add(w)).wrapping_mul(mul))
            .swap_bytes()
            .wrapping_add(g))
        .wrapping_mul(mul);
        let z = e.wrapping_add(f).wrapping_add(c);
        let a = ((x.wrapping_add(z)).wrapping_mul(mul).wrapping_add(y))
            .swap_bytes()
            .wrapping_add(b);
        let b = shift_mix(
            (z.wrapping_add(a))
                .wrapping_mul(mul)
                .wrapping_add(d)
                .wrapping_add(h),
        )
        .wrapping_mul(mul);
        b.wrapping_add(x)
    }

    fn weak_hash_len_32_with_seeds(&self, offset: usize, a: u64, b: u64) -> (u64, u64) {
        weak_hash_len_32_with_seeds(
            self.fetch64(offset),
            self.fetch64(offset + 8),
            self.fetch64(offset + 16),
            self.fetch64(offset + 24),
            a,
            b,
        )
    }
}

fn hash_len_16_u64(u: u64, v: u64) -> u64 {
    const MUL: u64 = 0x9ddfea08eb382d69;
    hash_len_16_with_mul(u, v, MUL)
}

fn hash_len_16_with_mul(u: u64, v: u64, mul: u64) -> u64 {
    // Murmur-inspired hashing.
    let mut a = (u ^ v).wrapping_mul(mul);
    a ^= a >> 47;
    let mut b = (v ^ a).wrapping_mul(mul);
    b ^= b >> 47;
    b.wrapping_mul(mul)
}

// Some primes between 2^63 and 2^64 for various uses.
const K0: u64 = 0xc3a5c85c97cb3127;
const K1: u64 = 0xb492b66fbe98f273;
const K2: u64 = 0x9ae16a3b2f90404f;

// Magic numbers for 32-bit hashing.  Copied from Murmur3.
const C1: u32 = 0xcc9e2d51;
const C2: u32 = 0x1b873593;

fn fmix(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

fn rotate32(val: u32, shift: u32) -> u32 {
    if shift == 0 {
        val
    } else {
        (val >> shift) | (val << (32 - shift))
    }
}
fn rotate64(val: u64, shift: u64) -> u64 {
    if shift == 0 {
        val
    } else {
        (val >> shift) | (val << (64 - shift))
    }
}

fn shift_mix(val: u64) -> u64 {
    val ^ (val >> 47)
}

fn permute3<T>(a: &mut T, b: &mut T, c: &mut T) {
    mem::swap(a, b);
    mem::swap(a, c);
}

fn mur(mut a: u32, mut h: u32) -> u32 {
    // Helper from Murmur3 for combining two 32-bit values.
    a = a.wrapping_mul(C1);
    a = rotate32(a, 17);
    a = a.wrapping_mul(C2);
    h ^= a;
    h = rotate32(h, 19);
    h.wrapping_mul(5).wrapping_add(0xe6546b64)
}

fn weak_hash_len_32_with_seeds(w: u64, x: u64, y: u64, z: u64, a: u64, b: u64) -> (u64, u64) {
    let a = a.wrapping_add(w);
    let b = rotate64(b.wrapping_add(a).wrapping_add(z), 21);
    let c = a;
    let a = a.wrapping_add(x);
    let a = a.wrapping_add(y);
    let b = b.wrapping_add(rotate64(a, 44));
    (a.wrapping_add(z), b.wrapping_add(c))
}

#[cfg(test)]
mod tests;

/// Hashes `data` using the [CityHash][cityhash] algorithm.
///
/// The exact implementation is decided upon by `T`:
///
/// |  `T`  | C++ Function Equivalent |
/// |-------|-------------------------|
/// | `u32` | `CityHash32`            |
/// | `u64` | `CityHash64`            |
///
/// ```rust
/// let hello: u32 = cityhasher::hash("hello");
/// let world: u64 = cityhasher::hash("world");
///
/// assert_eq!(hello, 2039911270);
/// assert_eq!(world, 16436542438370751598);
/// ```
///
/// [cityhash]: https://github.com/google/cityhash
#[inline]
pub fn hash<T>(data: impl AsRef<[u8]>) -> T
where
    T: FromCityHash,
{
    T::from_city_hash(data.as_ref())
}

/// A type that can be produced by the CityHash algorithm.
pub trait FromCityHash: sealed::Sealed {}

/// A type that can be produced by the CityHash algorithm using a seeded input.
pub trait FromSeededCityHash: sealed::SealedSeeded {}

mod sealed {
    pub trait Sealed {
        fn from_city_hash(data: &[u8]) -> Self;
    }

    pub trait SealedSeeded {
        fn from_city_hash_with_seed(data: &[u8], seed: Self) -> Self;
    }
}

impl FromCityHash for u32 {}

impl sealed::Sealed for u32 {
    #[inline]
    fn from_city_hash(data: &[u8]) -> Self {
        Input(data).hash32()
    }
}

impl FromCityHash for u64 {}

impl sealed::Sealed for u64 {
    #[inline]
    fn from_city_hash(data: &[u8]) -> Self {
        Input(data).hash64()
    }
}

impl FromSeededCityHash for u64 {}

impl sealed::SealedSeeded for u64 {
    #[inline]
    fn from_city_hash_with_seed(data: &[u8], seed: Self) -> Self {
        hash_len_16_u64(Input(data).hash64().wrapping_sub(K2), seed)
    }
}

/// Hashes `data` with a seed value, using the [CityHash][cityhash] algorithm.
///
/// The exact implementation is decided upon by `T`:
///
/// |  `T`  | C++ Function Equivalent |
/// |-------|-------------------------|
/// | `u64` | `CityHash64WithSeed`    |
///
/// ```rust
/// let hello: u64 = cityhasher::hash("hello");
/// let hello_with_seed: u64 = cityhasher::hash_with_seed("hello", 1);
///
/// assert_ne!(hello, hello_with_seed);
/// ```
///
/// [cityhash]: https://github.com/google/cityhash
#[inline]
pub fn hash_with_seed<T>(data: impl AsRef<[u8]>, seed: T) -> T
where
    T: FromSeededCityHash,
{
    T::from_city_hash_with_seed(data.as_ref(), seed)
}

/// A seedable [`Hasher`] and [`BuildHasher`] implementation using the
/// [CityHash][cityhash] algorithm.
///
/// This type can be used with any collections that support arbitrary
/// [`Hasher`]/[`BuildHasher`] types, including:
///
/// - [`std::collections::HashMap`]
/// - [`std::collections::HashSet`]
///
/// This crate provides type aliases for [`HashMap`] and [`HashSet`] that
/// utilize this type as the hasher.
///
/// # Using a Seeded Hasher with a HashMap
///
/// ```rust
/// use std::collections::HashMap;
///
/// use cityhasher::CityHasher;
///
/// let mut map = HashMap::with_hasher(CityHasher::with_seed(1));
/// map.insert(1, "hello");
/// assert!(map.contains_key(&1));
/// ```
///
/// [cityhash]: https://github.com/google/cityhash
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct CityHasher(Option<u64>);

impl CityHasher {
    /// Returns a new hasher with no seed.
    pub const fn new() -> Self {
        Self(None)
    }

    /// Returns a hasher that incorporates `seed` into the hashes produced.
    pub const fn with_seed(seed: u64) -> Self {
        Self(Some(seed))
    }
}

impl Hasher for CityHasher {
    fn finish(&self) -> u64 {
        self.0.unwrap_or(0)
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = Some(if let Some(seed) = self.0 {
            hash_with_seed(bytes, seed)
        } else {
            hash(bytes)
        });
    }
}

impl BuildHasher for CityHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        *self
    }
}

/// A type alias for [`std::collections::HashMap`] that hashes its keys using
/// [`CityHasher`].
#[cfg(feature = "std")]
pub type HashMap<K, V, S = CityHasher> = std::collections::HashMap<K, V, S>;
/// A type alias for [`std::collections::HashSet`] that hashes its members using
/// [`CityHasher`].
#[cfg(feature = "std")]
pub type HashSet<K, S = CityHasher> = std::collections::HashSet<K, S>;
