use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

pub struct SerializeMime<'a>(pub &'a ::mime::Mime);

impl<'a> Serialize for SerializeMime<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self.0)
    }
}

pub struct DeserializeMime(pub ::mime::Mime);

impl<'de> Deserialize<'de> for DeserializeMime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map(DeserializeMime)
            .map_err(de::Error::custom)
    }
}

pub mod mime {
    use super::{DeserializeMime, SerializeMime};

    use mime::Mime;
    use serde::{
        de::{Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    pub fn serialize<S>(mime: &Mime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializeMime(mime).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mime, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DeserializeMime::deserialize(deserializer)?.0)
    }
}

pub mod mime_vec_option {
    use super::{DeserializeMime, SerializeMime};

    use mime::Mime;
    use serde::{
        de::{Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    pub fn serialize<S>(opt: &Option<Vec<Mime>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        opt.as_ref()
            .map(|vec| vec.iter().map(SerializeMime).collect::<Vec<_>>())
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<Mime>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Option::<Vec<DeserializeMime>>::deserialize(deserializer)?
            .map(|vec| vec.into_iter().map(|mime| mime.0).collect()))
    }
}
