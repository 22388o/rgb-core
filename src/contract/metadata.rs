// RGB Core Library: a reference implementation of RGB smart contract standards.
// Written in 2019-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// To the extent possible under law, the author(s) have dedicated all copyright
// and related and neighboring rights to this software to the public domain
// worldwide. This software is distributed without any warranty.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

//! Convenience metadata accessor methods for Genesis and state transitions.

use std::collections::BTreeMap;

use amplify::Wrapper;

use super::data;
use crate::schema;

type MetadataInner = BTreeMap<schema::FieldType, Vec<data::Revealed>>;

/// Transition & genesis metadata fields
#[derive(Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, From)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Metadata(MetadataInner);

// TODO #107: Improve other iterators for contract collection types.
impl<'me> IntoIterator for &'me Metadata {
    type Item = (&'me schema::FieldType, &'me Vec<data::Revealed>);
    type IntoIter = std::collections::btree_map::Iter<'me, schema::FieldType, Vec<data::Revealed>>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct MetadataLeaf(pub schema::FieldType, pub data::Revealed);

#[cfg(test)]
mod test {
    use amplify::Wrapper;
    use bitcoin_hashes::Hash;
    use commit_verify::merkle::MerkleNode;
    use commit_verify::{merklize, CommitEncode};
    use secp256k1_zkp::rand::{thread_rng, RngCore};
    use strict_encoding::{StrictDecode, StrictEncode};
    use strict_encoding_test::test_vec_decoding_roundtrip;

    use super::*;
    //use lnpbp::commit_verify::CommitVerify;

    // Hard coded sample metadata object as shown below
    // Metadata({13: {U8(2), U8(3), U16(2), U32(2), U32(3),
    //    U64(2), U64(3), I8(2), I8(3), I32(2), I32(3),
    //    I64(2), I64(3), F32(2.0), F32(3.0), F64(2.0),
    //    F64(3.0), Bytes([1, 2, 3, 4, 5]), Bytes([10, 20, 30, 40, 50]),
    //    String("One Random String"), String("Another Random String")}})
    // It has Field_type = 13 with only single U16 and no I16 data types.
    static METADATA: [u8; 161] = include!("../../test/metadata.in");

    #[test]
    #[ignore]
    fn test_extraction() {
        let metadata = Metadata::strict_decode(&METADATA[..]).unwrap();

        let field_type = 13 as schema::FieldType;

        let field_1 = metadata.u8(field_type);
        let field_2 = metadata.u16(field_type);
        let field_3 = metadata.u32(field_type);
        let field_4 = metadata.u64(field_type);
        let field_5 = metadata.i8(field_type);
        let field_6 = metadata.i16(field_type);
        let field_7 = metadata.i32(field_type);
        let field_8 = metadata.i64(field_type);
        let field_9 = metadata.f32(field_type);
        let field_10 = metadata.f64(field_type);
        let field_11 = metadata.bytes(field_type);
        let field_12 = metadata.unicode_string(field_type);

        assert_eq!(field_1, vec![2, 3]);
        assert_eq!(field_2, vec![2]);
        assert_eq!(field_3, vec![2, 3]);
        assert_eq!(field_4, vec![2, 3]);
        assert_eq!(field_5, vec![2, 3]);
        assert_eq!(field_6, Vec::<i16>::new());
        assert_eq!(field_7, vec![2, 3]);
        assert_eq!(field_8, vec![2, 3]);
        assert_eq!(field_9, vec![2 as f32, 3 as f32]);
        assert_eq!(field_10, vec![2 as f64, 3 as f64]);
        assert_eq!(field_11, vec![
            [1u8, 2, 3, 4, 5].to_vec(),
            [10u8, 20, 30, 40, 50].to_vec()
        ]);
        assert_eq!(field_12, vec![
            "One Random String".to_string(),
            "Another Random String".to_string()
        ]);
    }

    #[test]
    #[ignore]
    fn test_encode_decode_meta() {
        let _: Metadata = test_vec_decoding_roundtrip(METADATA).unwrap();
    }

    #[test]
    #[ignore]
    #[should_panic(expected = "UnexpectedEof")]
    fn test_eof_metadata() {
        let mut data = METADATA.clone();
        data[0] = 0x36 as u8;
        Metadata::strict_decode(&data[..]).unwrap();
    }

    #[test]
    #[ignore]
    fn test_iteration_field() {
        let metadata = Metadata::strict_decode(&METADATA[..]).unwrap();
        let field_values = metadata.f32(13 as schema::FieldType);

        assert_eq!(field_values.into_iter().sum::<f32>(), 5f32);
    }

    #[test]
    fn test_commitencoding_field() {
        let mut rng = thread_rng();
        let mut data1 = Vec::new();
        data1.push(data::Revealed::U8(rng.next_u64() as u8));
        data1.push(data::Revealed::U16(rng.next_u64() as u16));
        data1.push(data::Revealed::U32(rng.next_u64() as u32));
        data1.push(data::Revealed::U64(rng.next_u64() as u64));

        let mut data2 = Vec::new();
        data2.push(data::Revealed::I8(rng.next_u64() as i8));
        data2.push(data::Revealed::I16(rng.next_u64() as i16));
        data2.push(data::Revealed::I32(rng.next_u64() as i32));
        data2.push(data::Revealed::I64(rng.next_u64() as i64));

        let mut byte_vec = vec![];
        for i in 0..10 {
            byte_vec.insert(i, rng.next_u32() as u8);
        }

        let mut data3 = Vec::new();
        data3.push(data::Revealed::F32(rng.next_u32() as f32));
        data3.push(data::Revealed::F64(rng.next_u32() as f64));
        data3.push(data::Revealed::Bytes(byte_vec));
        data3.push(data::Revealed::UnicodeString("Random String".to_string()));

        let field1 = 1 as schema::FieldType;
        let field2 = 2 as schema::FieldType;
        let field3 = 3 as schema::FieldType;

        let mut metadata_inner = BTreeMap::new();
        metadata_inner.insert(field1, data1.clone());
        metadata_inner.insert(field2, data2.clone());
        metadata_inner.insert(field3, data3.clone());

        let metadata = Metadata::from_inner(metadata_inner);

        let mut original_encoding = vec![];
        metadata
            .to_merkle_source()
            .consensus_commit()
            .commit_encode(&mut original_encoding);

        // Hand calculate the encoding
        // create the leaves
        let vec_1: Vec<(schema::FieldType, data::Revealed)> =
            data1.iter().map(|data| (field1, data.clone())).collect();
        let vec_2: Vec<(schema::FieldType, data::Revealed)> =
            data2.iter().map(|data| (field2, data.clone())).collect();
        let vec_3: Vec<(schema::FieldType, data::Revealed)> =
            data3.iter().map(|data| (field3, data.clone())).collect();

        // combine all the leaves
        let vec_4 = [vec_1, vec_2, vec_3].concat();

        // create MerkleNodes from each leaf
        let nodes: Vec<MerkleNode> = vec_4
            .iter()
            .map(|item| MerkleNode::hash(&StrictEncode::strict_serialize(item).unwrap()))
            .collect();

        // compute merkle root of all the nodes
        let (root, _) = merklize(MetadataLeaf::MERKLE_NODE_PREFIX, nodes);

        // Commit encode the root
        let handmade_encoding = root.commit_serialize();

        // This should match with original encoding
        assert_eq!(original_encoding, handmade_encoding);
    }
}
