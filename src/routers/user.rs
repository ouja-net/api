use actix_web::{get, web, HttpResponse};
use mongodb::{bson::doc, Client, Collection};
use serde_json::json;

use crate::models::Accounts;

#[get("{username}")]
pub async fn index(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<Accounts> = client.database("ouja_skins").collection("accounts");
    match collection
            .find_one(
                doc! { "username": { "$regex": "^".to_owned() + &username.to_lowercase() + "$", "$options": "i" } },
                None,
            )
            .await
        {
            Ok(Some(account)) => {
                let response = json!({
                    "id": account.id,
                    "username": account.username,
                    "skins": account.skins,
                    "about_me": account.about_me,
                    "profile_picture": account.profile_picture
                });
                HttpResponse::Ok()
                .json(json!({ "status": 200, "success": true, "user": response }))
            },
            Ok(None) => HttpResponse::NotFound()
                .json(json!({ "status": 404, "success": false, "error": "Not found" })),
            Err(err) => HttpResponse::InternalServerError()
                .json(json!({ "status": 500, "success": false, "error": err.to_string() })),
        }
}
