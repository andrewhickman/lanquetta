use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::FileSet;

impl Serialize for FileSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.to_bytes();
        let encoded = base64::encode(bytes);
        encoded.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FileSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        let bytes = base64::decode(encoded).map_err(Error::custom)?;
        FileSet::from_bytes(bytes).map_err(Error::custom)
    }
}
