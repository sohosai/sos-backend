use serde::Deserialize;

/// The decoded JWT claims returned from Firebase Authentication.
/// https://firebase.google.com/docs/rules/rules-and-auth
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub email: Option<String>,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub name: Option<String>,
    pub sub: String,
    // pub firebase.identities
    // pub firebase.sign_in_provider
}
