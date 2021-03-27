use mime::Mime;
use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileType(Mime);

impl FileType {
    pub fn from_mime(mime: Mime) -> Self {
        FileType(mime)
    }

    pub fn as_mime(&self) -> &'_ Mime {
        &self.0
    }

    pub fn into_mime(self) -> Mime {
        self.0
    }
}

impl Default for FileType {
    fn default() -> Self {
        FileType::from_mime(mime::APPLICATION_OCTET_STREAM)
    }
}

impl Serialize for FileType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self.as_mime())
    }
}

impl<'de> Deserialize<'de> for FileType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map(FileType::from_mime)
            .map_err(de::Error::custom)
    }
}
