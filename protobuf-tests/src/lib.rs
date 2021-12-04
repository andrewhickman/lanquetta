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
    use assert_json_diff::assert_json_eq;
    use prost::Message;
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
                "int64": 4,
                "uint32": 5,
                "uint64": 6,
                "sint32": 7,
                "sint64": 8,
                "fixed32": 9,
                "fixed64": 10,
                "sfixed32": 11,
                "sfixed64": 12,
                "bool": true,
                "string": "5",
                "bytes": "Ng==",
            })
        );
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
                "int64": [7, -8],
                "uint32": [9, 10],
                "uint64": [11, 12],
                "sint32": [13, -14],
                "sint64": [15, -16],
                "fixed32": [17, 18],
                "fixed64": [19, 20],
                "sfixed32": [21, -22],
                "sfixed64": [23, 24],
                "bool": [true, false],
                "string": ["25", "26"],
                "bytes": [base64::encode(b"27"), base64::encode(b"28")],
            })
        );
    }
}
