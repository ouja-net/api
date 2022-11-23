use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Accounts {
    pub date: DateTime,
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub session: Option<String>,
    pub about_me: Option<String>,
    pub profile_picture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinCollection {
    pub date: DateTime,
    pub id: String,
    pub hash: String,
    pub filename: String,
    pub title: String,
    pub description: String,
    pub size: usize,
    pub metadata: SkinMeta,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SkinMeta {
    Image {
        width: usize,
        height: usize,
        content_type: String,
    },
}
