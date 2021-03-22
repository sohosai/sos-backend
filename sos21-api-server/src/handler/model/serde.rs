pub mod mime {
    use mime::Mime;
    use serde::{
        de::{self, Deserialize, Deserializer},
        ser::Serializer,
    };

    pub fn serialize<S>(mime: &Mime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(mime)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mime, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
