// -*- mode: rust; -*-
//
// This file is part of schnorrkel.
// Copyright (c) 2019 Web 3 Foundation
// See LICENSE for licensing information.
//
// Authors:
// - Jeff Burdges <jeff@web3.foundation>

//! ristretto point tooling
//! 
//! We provide a `RistrettoBoth` type that contains both an uncompressed
//! `RistrettoPoint` along side its matching `CompressedRistretto`, 
//! which helps several protocols avoid duplicate ristretto compressions
//! and/or decompressions.  

// We expect some ariant to eventually appear in curve25519-dalek, ala
// ..


use core::fmt::{Debug};

use curve25519_dalek::ristretto::{CompressedRistretto,RistrettoPoint};
// use curve25519_dalek::scalar::Scalar;

use crate::errors::{SignatureError,SignatureResult};


/// Compressed Ristretto point length
pub const RISTRETTO_POINT_LENGTH: usize = 32;


/// A `RistrettoBoth` contains both an uncompressed `RistrettoPoint`
/// as well as the corresponding `CompressedRistretto`.  It provides
/// a convenient middle ground for protocols that both hash compressed
/// points to derive scalars for use with uncompressed points.
#[derive(Copy, Clone, Default, Eq)]  // PartialEq optimnized below
pub struct RistrettoBoth {
    compressed: CompressedRistretto,
    point: RistrettoPoint,
}

impl Debug for RistrettoBoth {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "RistrettoPoint( {:?} )", self.compressed)
    }
}

impl RistrettoBoth {
    const DESCRIPTION : &'static str = "A ristretto point represented as a 32-byte compressed point";

    // I dislike getter methods, and prefer direct field access, but doing
    // getters here permits the fields being private, and gives us faster
    // equality comparisons.

    /// Access the compressed Ristretto form
    pub fn as_compressed(&self) -> &CompressedRistretto { &self.compressed }

    /// Extract the compressed Ristretto form
    pub fn into_compressed(self) -> CompressedRistretto { self.compressed }

    /// Access the point form
    pub fn as_point(&self) -> &RistrettoPoint { &self.point }

    /// Extract the point form
    pub fn into_point(self) -> RistrettoPoint { self.point }

    /// Decompress into the `RistrettoBoth` format that also retains the
    /// compressed form.
    pub fn from_compressed(compressed: CompressedRistretto) -> SignatureResult<RistrettoBoth> {
        Ok(RistrettoBoth {
            point: compressed.decompress().ok_or(SignatureError::PointDecompressionError) ?,
            compressed,
        })
    }

    /// Compress into the `RistrettoBoth` format that also retains the
    /// uncompressed form.
    pub fn from_point(point: RistrettoPoint) -> RistrettoBoth {
        RistrettoBoth {
            compressed: point.compress(),
            point,
        }
    }

    /// Convert this public key to a byte array.
    #[inline]
    pub fn to_bytes(&self) -> [u8; RISTRETTO_POINT_LENGTH] {
        self.compressed.to_bytes()
    }

    /// Construct a `RistrettoBoth` from a slice of bytes.
    ///
    /// # Warning
    ///
    /// The caller is responsible for ensuring that the bytes passed into this
    /// method actually represent a `curve25519_dalek::ristretto::CompressedRistretto`
    /// and that said compressed point is actually a point on the curve.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate schnorrkel;
    /// #
    /// use schnorrkel::points::RistrettoBoth;
    /// use schnorrkel::PUBLIC_KEY_LENGTH;
    /// use schnorrkel::SignatureError;
    ///
    /// # fn doctest() -> Result<RistrettoBoth, SignatureError> {
    /// let public_key_bytes: [u8; PUBLIC_KEY_LENGTH] = [
    ///    215,  90, 152,   1, 130, 177,  10, 183, 213,  75, 254, 211, 201, 100,   7,  58,
    ///     14, 225, 114, 243, 218, 166,  35,  37, 175,   2,  26, 104, 247,   7,   81, 26];
    ///
    /// let public_key = RistrettoBoth::from_bytes(&public_key_bytes)?;
    /// #
    /// # Ok(public_key)
    /// # }
    /// #
    /// # fn main() {
    /// #     doctest();
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// A `Result` whose okay value is an EdDSA `RistrettoBoth` or whose error value
    /// is an `SignatureError` describing the error that occurred.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> SignatureResult<RistrettoBoth> {
        RistrettoBoth::from_bytes_ser("RistrettoPoint",RistrettoBoth::DESCRIPTION,bytes)
    }

    /// Variant of `RistrettoBoth::from_bytes` that propogates more informative errors.
    #[inline]
    pub fn from_bytes_ser(name: &'static str, description: &'static str, bytes: &[u8]) -> SignatureResult<RistrettoBoth> {
        if bytes.len() != RISTRETTO_POINT_LENGTH {
            return Err(SignatureError::BytesLengthError{
                name, description, length: RISTRETTO_POINT_LENGTH,
            });
        }
        let mut compressed = CompressedRistretto([0u8; RISTRETTO_POINT_LENGTH]);
        compressed.0.copy_from_slice(&bytes[..32]);
        RistrettoBoth::from_compressed(compressed)
    }
}

serde_boilerplate!(RistrettoBoth);

/// We hide fields largely so that only compairing the compressed forms works.
impl PartialEq<Self> for RistrettoBoth {
    fn eq(&self, other: &Self) -> bool {
        let r = self.compressed.eq(&other.compressed);
        debug_assert_eq!(r, self.point.eq(&other.point));
        r
    }

    // fn ne(&self, other: &Rhs) -> bool {
    //   self.compressed.0.ne(&other.compressed.0)
    // }
}

// impl Eq for RistrettoBoth {}

impl PartialOrd<RistrettoBoth> for RistrettoBoth {
    fn partial_cmp(&self, other: &RistrettoBoth) -> Option<::core::cmp::Ordering> {
        self.compressed.0.partial_cmp(&other.compressed.0)
    }

    // fn lt(&self, other: &Rhs) -> bool {
    //    self.compressed.0.lt(&other.compressed.0)
    // }
    // fn le(&self, other: &Rhs) -> bool {
    //    self.compressed.0.le(&other.compressed.0)
    // }
    // fn gt(&self, other: &Rhs) -> bool {
    //    self.compressed.0.gt(&other.compressed.0)
    // }
    // fn ge(&self, other: &Rhs) -> bool {
    //    self.compressed.0.ge(&other.compressed.0)
    // }
}

impl Ord for RistrettoBoth {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.compressed.0.cmp(&other.compressed.0)
    }

    // fn max(self, other: Self) -> Self {
    //    self.compressed.0.max(other.compressed.0)
    // }
    // fn min(self, other: Self) -> Self {
    //    self.compressed.0.min(other.compressed.0)
    // }
}

impl ::core::hash::Hash for RistrettoBoth {
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        self.compressed.0.hash(state);
    }
}

