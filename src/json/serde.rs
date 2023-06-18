use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::JsonText;

impl Serialize for JsonText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.original_data())
    }
}

pub fn deserialize_short<'de, D>(deserializer: D) -> Result<JsonText, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(JsonText::short(s))
}

pub fn deserialize_short_opt<'de, D>(deserializer: D) -> Result<Option<JsonText>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(s.map(JsonText::short))
}
