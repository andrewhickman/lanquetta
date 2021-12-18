use grpc_client_protobuf::FileSet;
use once_cell::sync::Lazy;

pub mod definitions {
    include!(concat!(env!("OUT_DIR"), "/test.rs"));
}

pub static TYPE_MAP: Lazy<FileSet> = Lazy::new(|| {
    FileSet::from_bytes(
        include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin")).as_ref(),
    )
    .unwrap()
});

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use assert_json_diff::assert_json_eq;
    use prost::{encoding::WireType, Message};
    use serde_json::{json, Value};

    use crate::{definitions, TYPE_MAP};

    #[test]
    fn test_scalars() {
        let scalars = definitions::Scalars {
            double: 1.1,
            float: 2.2,
            int32: 3,
            int64: 4,
            uint32: 5,
            uint64: 6,
            sint32: 7,
            sint64: 8,
            fixed32: 9,
            fixed64: 10,
            sfixed32: 11,
            sfixed64: 12,
            r#bool: true,
            string: "5".to_owned(),
            bytes: b"6".to_vec(),
        };
        let bytes = scalars.encode_to_vec();

        let value = TYPE_MAP.get_message_by_name(".test.Scalars").unwrap();
        let actual: Value = serde_json::from_str(&value.decode(&bytes).unwrap()).unwrap();

        assert_json_eq!(
            actual,
            json!({
                "double": 1.1,
                "float": 2.2f32,
                "int32": 3,
                "int64": "4",
                "uint32": 5,
                "uint64": "6",
                "sint32": 7,
                "sint64": "8",
                "fixed32": 9,
                "fixed64": "10",
                "sfixed32": 11,
                "sfixed64": "12",
                "bool": true,
                "string": "5",
                "bytes": "Ng==",
            })
        );
    }

    #[test]
    fn test_extra_fields() {
        let mut bytes = vec![];
        prost::encoding::encode_key(100, WireType::Varint, &mut bytes);
        prost::encoding::encode_varint(42, &mut bytes);

        let value = TYPE_MAP.get_message_by_name(".test.Scalars").unwrap();
        let actual: Value = serde_json::from_str(&value.decode(&bytes).unwrap()).unwrap();

        assert_json_eq!(actual, json!({}));
    }

    #[test]
    fn test_scalar_arrays() {
        let scalars = definitions::ScalarArrays {
            double: vec![1.1, 2.2],
            float: vec![3.3f32, 4.4f32],
            int32: vec![5, -6],
            int64: vec![7, -8],
            uint32: vec![9, 10],
            uint64: vec![11, 12],
            sint32: vec![13, -14],
            sint64: vec![15, -16],
            fixed32: vec![17, 18],
            fixed64: vec![19, 20],
            sfixed32: vec![21, -22],
            sfixed64: vec![23, 24],
            r#bool: vec![true, false],
            string: vec!["25".to_owned(), "26".to_owned()],
            bytes: vec![b"27".to_vec(), b"28".to_vec()],
        };
        let bytes = scalars.encode_to_vec();

        let value = TYPE_MAP.get_message_by_name(".test.ScalarArrays").unwrap();
        let actual: Value = serde_json::from_str(&value.decode(&bytes).unwrap()).unwrap();

        assert_json_eq!(
            actual,
            json!({
                "double": [1.1, 2.2],
                "float": [3.3f32, 4.4f32],
                "int32": [5, -6],
                "int64": ["7", "-8"],
                "uint32": [9, 10],
                "uint64": ["11", "12"],
                "sint32": [13, -14],
                "sint64": ["15", "-16"],
                "fixed32": [17, 18],
                "fixed64": ["19", "20"],
                "sfixed32": [21, -22],
                "sfixed64": ["23", "24"],
                "bool": [true, false],
                "string": ["25", "26"],
                "bytes": [base64::encode(b"27"), base64::encode(b"28")],
            })
        );
    }

    #[test]
    fn test_complex_type() {
        let scalars = definitions::ComplexType {
            string_map: HashMap::from([
                (
                    "1".to_owned(),
                    definitions::Scalars {
                        double: 1.1,
                        float: 2.2,
                        int32: 3,
                        ..Default::default()
                    },
                ),
                (
                    "2".to_owned(),
                    definitions::Scalars {
                        int64: 4,
                        uint32: 5,
                        uint64: 6,
                        ..Default::default()
                    },
                ),
            ]),
            int_map: HashMap::from([
                (
                    3,
                    definitions::Scalars {
                        sint32: 7,
                        sint64: 8,
                        fixed32: 9,
                        ..Default::default()
                    },
                ),
                (
                    4,
                    definitions::Scalars {
                        sint64: 8,
                        fixed32: 9,
                        fixed64: 10,
                        ..Default::default()
                    },
                ),
            ]),
            nested: Some(definitions::Scalars {
                sfixed32: 11,
                sfixed64: 12,
                r#bool: true,
                string: "5".to_owned(),
                bytes: b"6".to_vec(),
                ..Default::default()
            }),
            my_enum: vec![0, 1, 2, 3],
        };
        let bytes = scalars.encode_to_vec();

        let value = TYPE_MAP.get_message_by_name(".test.ComplexType").unwrap();
        let actual: Value = serde_json::from_str(&value.decode(&bytes).unwrap()).unwrap();

        assert_json_eq!(
            actual,
            json!({
                "stringMap": {
                    "1": {
                        "double": 1.1,
                        "float": 2.2f32,
                        "int32": 3,
                    },
                    "2": {
                        "int64": "4",
                        "uint32": 5,
                        "uint64": "6",
                    },
                },
                "intMap": {
                    "3": {
                        "sint32": 7,
                        "sint64": "8",
                        "fixed32": 9,
                    },
                    "4": {
                        "sint64": "8",
                        "fixed32": 9,
                        "fixed64": "10",
                    },
                },
                "nested": {
                    "sfixed32": 11,
                    "sfixed64": "12",
                    "bool": true,
                    "string": "5",
                    "bytes": "Ng==",
                },
                "myEnum": ["DEFAULT", "FOO", 2, "BAR"],
            })
        );
    }

    #[test]
    fn test_well_known_types() {
        let scalars = definitions::WellKnownTypes {
            timestamp: Some(prost_types::Timestamp {
                seconds: 63_108_020,
                nanos: 21_000_000,
            }),
            duration: Some(prost_types::Duration {
                seconds: 1,
                nanos: 340_012,
            }),
            r#struct: Some(prost_types::Struct {
                fields: BTreeMap::from([
                    (
                        "number".to_owned(),
                        prost_types::Value {
                            kind: Some(prost_types::value::Kind::NumberValue(42.)),
                        },
                    ),
                    (
                        "null".to_owned(),
                        prost_types::Value {
                            kind: Some(prost_types::value::Kind::NullValue(0)),
                        },
                    ),
                ]),
            }),
            float: Some(42.1),
            double: Some(12.4),
            int32: Some(1),
            int64: Some(-2),
            uint32: Some(3),
            uint64: Some(4),
            bool: Some(false),
            string: Some("hello".to_owned()),
            bytes: Some(b"hello".to_vec()),
            mask: Some(prost_types::FieldMask {
                paths: vec!["field_one".to_owned(), "field_two.b.d".to_owned()],
            }),
            list: Some(prost_types::ListValue {
                values: vec![
                    prost_types::Value {
                        kind: Some(prost_types::value::Kind::StringValue("foo".to_owned())),
                    },
                    prost_types::Value {
                        kind: Some(prost_types::value::Kind::BoolValue(false)),
                    },
                ],
            }),
            null: 0,
            empty: Some(()),
        };
        let bytes = scalars.encode_to_vec();

        let value = TYPE_MAP
            .get_message_by_name(".test.WellKnownTypes")
            .unwrap();
        let actual: Value = serde_json::from_str(&value.decode(&bytes).unwrap()).unwrap();

        assert_json_eq!(
            actual,
            json!({
                "timestamp": "1972-01-01T10:00:20.021Z",
                "duration": "1.000340012s",
                "struct": {
                    "number": 42.0,
                    "null": null,
                },
                "float": 42.1f32,
                "double": 12.4,
                "int32": 1,
                "int64": "-2",
                "uint32": 3,
                "uint64": "4",
                "bool": false,
                "string": "hello",
                "bytes": "aGVsbG8=",
                "mask": "fieldOne,fieldTwo.b.d",
                "list": ["foo", false],
                "empty": {}
            })
        );
    }
}
