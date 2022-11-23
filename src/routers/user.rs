use actix_web::{get, web, HttpResponse};
use bson::DateTime;
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::Accounts;
use futures_util::stream::StreamExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct RespondSkin {
    pub id: String,
    pub date: DateTime,
    pub title: String,
    pub description: String,
    pub owner: String,
}

#[get("/{username}/skins")]
pub async fn get_user_skins(
    client: web::Data<Client>,
    username: web::Path<String>,
) -> HttpResponse {
    let username = username.into_inner();
    let user_collection: Collection<Accounts> =
        client.database("ouja_skins").collection("accounts");
    let skin_collection: Collection<RespondSkin> =
        client.database("ouja_skins").collection("skins");
    match user_collection.find_one(doc! { "username": { "$regex": "^".to_owned() + &username.to_lowercase() + "$", "$options": "i" } }, None).await {
        Ok(Some(user)) => {
            let mut skins = skin_collection.find(doc! { "owner": &user.id }, None).await;
            let mut results: Vec<RespondSkin> = Vec::new();

            while let Some(skin) = skins.as_mut().unwrap().next().await {
                match skin {
                    Ok(skin) => {
                        results.push(skin);
                    }
                    Err(err) => {
                        println!("{:?} - collecting skins", err);
                        return HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() }));
                    }
                }
            }

            HttpResponse::Ok().json(json!(results))
        },
        Ok(None) => HttpResponse::NotFound().json(json!({ "status": 404, "success": false, "error": "User not found" })),
        Err(err) => {
            println!("{:?} - finding user", err);
            HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() }))
        }
    }
}

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
                    "about_me": account.about_me,
                    "profile_picture": account.profile_picture
                });
                HttpResponse::Ok().json(json!(response))
            },
            Ok(None) => HttpResponse::NotFound()
                .json(json!({ "status": 404, "success": false, "error": "Not found" })),
            Err(err) => HttpResponse::InternalServerError()
                .json(json!({ "status": 500, "success": false, "error": err.to_string() })),
        }
}
