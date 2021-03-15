use mime::Mime;

#[derive(Debug, Clone)]
pub struct FileType(Mime);

impl FileType {
    pub fn from_mime(mime: Mime) -> Self {
        FileType(mime)
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
