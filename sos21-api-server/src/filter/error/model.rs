use serde::{ser::Serializer, Serialize};
use warp::http::StatusCode;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthenticationErrorId {
    Unauthrized,
    InvalidToken,
    UnverifiedEmail,
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
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum ErrorBody {
    Api { info: serde_json::Value },
    Authentication { id: AuthenticationErrorId },
    Request { id: RequestErrorId },
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
