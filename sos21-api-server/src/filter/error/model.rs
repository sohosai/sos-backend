use serde::{ser::Serializer, Serialize};
use warp::http::StatusCode;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthenticationErrorId {
    Unauthorized,
    InvalidToken,
    InvalidEmailAddress,
    UnverifiedEmailAddress,
    NotUniversityEmailAddress,
    NoEmail,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestErrorId {
    NotFound,
    UnsupportedMediaType,
    MissingHeader,
    MethodNotAllowed,
    InvalidHeader,
    InvalidQuery,
    InvalidBody,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum ErrorBody {
    Api { info: serde_json::Value },
    Authentication { id: AuthenticationErrorId },
    Request { id: RequestErrorId },
    NotSignedUp,
    Internal,
}

#[derive(Debug, Clone, Serialize)]
pub struct Error {
    pub error: ErrorBody,
    #[serde(serialize_with = "serialize_status_code")]
    pub status: StatusCode,
}

fn serialize_status_code<S>(code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u16(code.as_u16())
}
