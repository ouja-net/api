use actix_web::{get, post, put, web, HttpRequest, HttpResponse, patch};
use mongodb::{bson::{doc,  DateTime}, Client, Collection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    magic_crypt::{decrypt, encrypt},
    models::{Accounts, Skins},
};

#[derive(Serialize, Deserialize)]
pub struct LoginParams {
    email: String,
    password: String,
    remember_me: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterParams {
    username: String,
    email: String,
    password: String,
    conf_password: String,
    agreed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RespondAccount {
    id: String,
    email: String,
    session: String,
    username: String,
    about_me: String,
    profile_picture: String,
    skins: Vec<Skins>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEmailParams {
    email: String
}

fn get_session_token<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    return req.headers().get("x-session")?.to_str().ok();
}

#[get("/@me")]
async fn me(client: web::Data<Client>, req: HttpRequest) -> HttpResponse {
    if let Some(token) = get_session_token(&req) {
        let collection: Collection<Accounts> = client.database("ouja_skins").collection("accounts");
        match collection.find_one(doc! { "session": token }, None).await {
            Ok(Some(account)) => {
                let respond = json!({
                    "id": account.id,
                    "email": decrypt(&account.email),
                    "date": account.date,
                    "session": account.session,
                    "username": account.username,
                    "about_me": account.about_me,
                    "profile_picture": account.profile_picture,
                    "skins": account.skins
                });
                HttpResponse::Ok()
                    .json(json!({ "status": 200, "success": true, "account": respond }))
            }
            Ok(None) => HttpResponse::NotFound()
                .json(json!({ "status": 404, "success": false, "error": "Session not found." })),
            Err(err) => {
                println!("{}", err);
                HttpResponse::InternalServerError()
                    .json(json!({ "status": 500, "success": false, "error": err.to_string() }))
            }
        }
    } else {
        HttpResponse::Unauthorized().json(json!({ "status": 401, "success": false }))
    }
}

#[put("/register")]
async fn register(client: web::Data<Client>, params: web::Form<RegisterParams>) -> HttpResponse {
    let collection: Collection<Accounts> = client.database("ouja_skins").collection("accounts");
    match collection.find_one(doc! { "username": { "$regex": "^".to_owned() + &params.username.to_lowercase() + "$", "$options": "i" } }, None).await {
        Err(err) => HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() })),
        Ok(Some(_account)) => 
        HttpResponse::Ok()
            .json(json!({ "status": 200, "success": false, "error": "Name already exists!" })),
            Ok(None) => {
                match collection
                .find_one(doc! { "email": encrypt(&params.email) }, None)
                .await {
                    Err(err) => HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() })),
                    Ok(Some(_account)) => {
                        HttpResponse::Ok()
                    .json(json!({ "status": 200, "success": false, "error": "Email already exists!" }))
                    },
                    Ok(None) => {
                        if params.password != params.conf_password {
                            HttpResponse::Ok().json(json!({ "status": 200, "success": false, "error": "Password does not match" }))
                        } else {
                            let new_doc = Accounts {
                                date            : DateTime::now(),
                                id              : Uuid::new_v4().to_string(),
                                username        : params.username.to_string(),
                                email           : encrypt(&params.email.to_lowercase()),
                                password        : encrypt(&params.password),
                                session         : None,
                                about_me        : None,
                                profile_picture : None,
                                skins           : None
                            };

                            // HttpResponse::Ok().json(json!({ "status": 200, "success": true }))
                            match collection.insert_one(&new_doc, None).await {
                                Ok(_result) => HttpResponse::Ok().json(json!({ "status": 200, "success": true })),
                                Err(err) => HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() }))
                            }
                            // create account here
                            
                        }
                    }
                }
            }
    }
}

#[patch("/email")]
async fn update_email(client: web::Data<Client>, req: HttpRequest, params: web::Form<UpdateEmailParams>) -> HttpResponse {
    if let Some(token) = get_session_token(&req) {
        let collection: Collection<Accounts> = client.database("ouja_skins").collection("accounts");
        match collection.find_one(doc! { "session": token }, None).await {
            Ok(Some(account)) => {
                match collection.update_one(doc! { "id": account.id }, doc! { "$set": { "email": encrypt(&params.email) } }, None).await {
                    Ok(_update_result) => {
                        HttpResponse::Ok().json(json!({ "status": 200, "success": true }))
                    },
                    Err(err) => {
                        HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() }))
                    }
                }
            },
            Ok(None) => HttpResponse::Unauthorized().json(json!({ "status": 401, "success": false, "error": "Could not authenticate request." })),
            Err(err) => HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({ "status": 401, "success": false, "error": "Could not authenticate request." }))
    }
}

#[post("/login")]
async fn login(client: web::Data<Client>, params: web::Form<LoginParams>) -> HttpResponse {
    let collection: Collection<Accounts> = client.database("ouja_skins").collection("accounts");
    let email = encrypt(&params.email);
    let password = encrypt(&params.password);
    let session_id = encrypt(&Uuid::new_v4().to_string());

    match collection
        .find_one(doc! { "email": &email, "password": &password }, None)
        .await
    {
        Ok(Some(account)) => {
            match collection
                .update_one(
                    doc! { "id": account.id },
                    doc! { "$set": { "session": session_id.clone() } },
                    None,
                )
                .await
            {
                Ok(_update_result) => HttpResponse::Ok()
                    .json(json!({ "code": 200, "success": true, "ID": session_id })),

                Err(err) => HttpResponse::InternalServerError()
                    .json(json!({"code": 500, "success": false, "error": err.to_string()})),
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(json!({ "code": 404, "success": false, "error": "Account not found." }))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
