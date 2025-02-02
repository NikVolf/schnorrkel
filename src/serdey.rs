// -*- mode: rust; -*-
//
// This file is part of schnorrkel.
// Copyright (c) 2019 Isis Lovecruft and Web 3 Foundation
// See LICENSE for licensing information.
//
// Authors:
// - Isis Agora Lovecruft <isis@patternsinthevoid.net>
// - Jeff Burdges <jeff@web3.foundation>

//! Various test tooling related to serde
//!
//! Right now, these

// #[cfg(feature = "serde")]
// use serde::{Serialize, Deserialize};
// #[cfg(feature = "serde")]
// use serde::{Serializer, Deserializer};
// #[cfg(feature = "serde")]
// use serde::de::Error as SerdeError;
// #[cfg(feature = "serde")]
// use serde::de::Visitor;


#[cfg(feature = "serde")]
macro_rules! serde_boilerplate { ($t:ty) => {
impl ::serde::Serialize for $t {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
        serializer.serialize_bytes(&self.to_bytes()[..])
    }
}

impl<'d> ::serde::Deserialize<'d> for $t {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: ::serde::Deserializer<'d> {
        struct MyVisitor;

        impl<'d> ::serde::de::Visitor<'d> for MyVisitor {
            type Value = $t;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter.write_str(Self::Value::DESCRIPTION)
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<$t, E> where E: ::serde::de::Error {
                Self::Value::from_bytes(bytes).map_err(crate::errors::serde_error_from_signature_error)
            }
        }
        deserializer.deserialize_bytes(MyVisitor)
    }
}
} } // macro_rules! serde_boilerplate

#[cfg(not(feature = "serde"))]
macro_rules! serde_boilerplate { ($t:ty) => { } }

#[cfg(all(test, feature = "serde"))]
mod test {
    use std::vec::Vec;

    use bincode::{serialize, serialized_size, deserialize, Infinite};

    use curve25519_dalek::ristretto::{CompressedRistretto};

    use crate::*;

    static COMPRESSED_PUBLIC_KEY : CompressedRistretto = CompressedRistretto([
        208, 120, 140, 129, 177, 179, 237, 159,
        252, 160, 028, 013, 206, 005, 211, 241,
        192, 218, 001, 097, 130, 241, 020, 169,
        119, 046, 246, 029, 079, 080, 077, 084]);

    /*
    static ED25519_PUBLIC_KEY: CompressedEdwardsY = CompressedEdwardsY([
        130, 039, 155, 015, 062, 076, 188, 063,
        124, 122, 026, 251, 233, 253, 225, 220,
        014, 041, 166, 120, 108, 035, 254, 077,
        160, 083, 172, 058, 219, 042, 086, 120, ]);
    */

    static ED25519_SECRET_KEY: MiniSecretKey = MiniSecretKey([
        062, 070, 027, 163, 092, 182, 011, 003,
        077, 234, 098, 004, 011, 127, 079, 228,
        243, 187, 150, 073, 201, 137, 076, 022,
        085, 251, 152, 002, 241, 042, 072, 054, ]);

    /// Ed25519 signature with the above keypair of a blank message.
    static SIGNATURE_BYTES: [u8; SIGNATURE_LENGTH] = [
        010, 126, 151, 143, 157, 064, 047, 001,
        196, 140, 179, 058, 226, 152, 018, 102,
        160, 123, 080, 016, 210, 086, 196, 028,
        053, 231, 012, 157, 169, 019, 158, 063,
        045, 154, 238, 007, 053, 185, 227, 229,
        079, 108, 213, 080, 124, 252, 084, 167,
        216, 085, 134, 144, 129, 149, 041, 081,
        063, 120, 126, 100, 092, 059, 050, 011, ];


    #[test]
    fn serialize_deserialize_signature() {
        let signature: Signature = Signature::from_bytes(&SIGNATURE_BYTES).unwrap();
        let encoded_signature: Vec<u8> = serialize(&signature, Infinite).unwrap();
        let decoded_signature: Signature = deserialize(&encoded_signature).unwrap();

        assert_eq!(signature, decoded_signature);
    }

    #[test]
    fn serialize_deserialize_public_key() {
        let public_key = PublicKey::from_compressed(COMPRESSED_PUBLIC_KEY).unwrap();
        let encoded_public_key: Vec<u8> = serialize(&public_key, Infinite).unwrap();
        let decoded_public_key: PublicKey = deserialize(&encoded_public_key).unwrap();

        assert_eq!(public_key, decoded_public_key);
    }

    /*
    TODO: Actually test serde on real secret key, not just mini one.
    fn serialize_deserialize_secret_key() {
        let encoded_secret_key: Vec<u8> = serialize(&SECRET_KEY, Infinite).unwrap();
        let decoded_secret_key: MiniSecretKey = deserialize(&encoded_secret_key).unwrap();

        for i in 0..64 {
            assert_eq!(ED25519_SECRET_KEY.0[i], decoded_secret_key.0[i]);
        }
    }
    */

    #[test]
    fn serialize_deserialize_mini_secret_key() {
        let encoded_secret_key: Vec<u8> = serialize(&ED25519_SECRET_KEY, Infinite).unwrap();
        let decoded_secret_key: MiniSecretKey = deserialize(&encoded_secret_key).unwrap();

        for i in 0..32 {
            assert_eq!(ED25519_SECRET_KEY.0[i], decoded_secret_key.0[i]);
        }
    }

    #[test]
    fn serialize_public_key_size() {
        let public_key = PublicKey::from_compressed(COMPRESSED_PUBLIC_KEY).unwrap();
        assert_eq!(serialized_size(&public_key) as usize, 32+8);  // Size specific to bincode==1.0.1
    }

    #[test]
    fn serialize_signature_size() {
        let signature: Signature = Signature::from_bytes(&SIGNATURE_BYTES).unwrap();
        assert_eq!(serialized_size(&signature) as usize, 64+8);  // Size specific to bincode==1.0.1
    }

    #[test]
    fn serialize_secret_key_size() {
        assert_eq!(serialized_size(&ED25519_SECRET_KEY) as usize, 32+8);
        let secret_key = ED25519_SECRET_KEY.expand();
        assert_eq!(serialized_size(&secret_key) as usize, 64+8);  // Sizes specific to bincode==1.0.1
    }
}
