use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Accounts {
    pub date: DateTime,
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub session: Option<String>,
    pub about_me: Option<String>,
    pub profile_picture: Option<String>,
    pub skins: Option<Vec<Skins>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Skins {
    pub id: String,
}
