use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{SerializeSeq, Serializer},
};
pub mod wordlist {

    use super::*;

    pub fn ser<S: Serializer>(value: &Vec<(String, String)>, s: S) -> Result<S::Ok, S::Error> {
        // Format as an array of key:value strings
        let mut seq = s.serialize_seq(Some(value.len()))?;
        for (key, value) in value {
            let formatted = format!("{}:{}", key, value);
            seq.serialize_element(&formatted)?;
        }
        seq.end()
    }

    pub fn de<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<(String, String)>, D::Error> {
        // Deserialize as an array of key:value strings
        let vec = Vec::<String>::deserialize(d)?;
        let mut result = Vec::with_capacity(vec.len());
        for item in vec {
            let parts: Vec<&str> = item.split(':').collect();
            if parts.len() != 2 {
                return Err(de::Error::custom("Invalid format"));
            }
            result.push((parts[0].to_string(), parts[1].to_string()));
        }
        Ok(result)
    }
}
