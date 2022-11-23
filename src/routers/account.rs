use actix_web::{get, post, put, web, HttpRequest, HttpResponse, patch};
use mongodb::{bson::{doc,  DateTime}, Client, Collection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    util::{get_session_token, verified_csrf},
    magic_crypt::{decrypt, encrypt},
    models::Accounts,
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
    profile_picture: String
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEmailParams {
    email: String
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUserParams {
    username: String,
    about_me: String
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
                    "profile_picture": account.profile_picture
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
async fn register(client: web::Data<Client>, req: HttpRequest, params: web::Form<RegisterParams>) -> HttpResponse {
    if !verified_csrf(&req) {
        return HttpResponse::Unauthorized().json(json!({ "status": 401, "success": false, "error": "Invalid CSRF Token!" }));
    }
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
                            if params.username.len() > 16 {
                                return HttpResponse::Ok().json(json!({ "stauts": 200, "success": false, "error": "Username is too long!" }))
                            }
                            if params.email.len() > 256 {
                                return HttpResponse::Ok().json(json!({ "stauts": 200, "success": false, "error": "Email is too long!" }))
                            }
                            // Limiting the lenght because it could take a lot of space in the database.
                            if params.password.len() > 256 {
                                return HttpResponse::Ok().json(json!({ "stauts": 200, "success": false, "error": "Password is too long!" }))
                            }
                            let new_doc = Accounts {
                                date: DateTime::now(),
                                id: Uuid::new_v4().to_string(),
                                username: params.username.to_string(),
                                email: encrypt(&params.email.to_lowercase()),
                                password: encrypt(&params.password),
                                session: None,
                                about_me: None,
                                profile_picture : None
                            };
                            match collection.insert_one(&new_doc, None).await {
                                Ok(_result) => HttpResponse::Ok().json(json!({ "status": 200, "success": true })),
                                Err(err) => HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() }))
                            }                            
                        }
                    }
                }
            }
    }
}

#[patch("")]
async fn update_user(client: web::Data<Client>, req: HttpRequest, params: web::Form<UpdateUserParams>) -> HttpResponse {
    if !verified_csrf(&req) {
        return HttpResponse::Unauthorized().json(json!({ "status": 401, "success": false, "error": "Invalid CSRF Token!" }));
    }
    if let Some(token) = get_session_token(&req) {
        let collection: Collection<Accounts> = client.database("ouja_skins").collection("accounts");
        match collection.find_one(doc! { "session": token }, None).await {
            Ok(Some(account)) => {
                if params.about_me.len() > 256 {
                    return HttpResponse::Ok().json(json!({ "code": 200, "success": false, "error": "About me is too long!" }))
                }
                if params.username.len() > 16 {
                    return HttpResponse::Ok().json(json!({ "code": 200, "success": false, "error": "Username is too long!" }))
                }
                match collection.update_one(doc! { "id": account.id }, doc! { "$set": { "username": &params.username, "about_me": &params.about_me } }, None).await {
                    Ok(_update_result) => {
                        HttpResponse::Ok().json(json!({ "code": 200, "success": true, "account": doc! { "username": &params.username, "about_me": &params.about_me } }))
                    },
                    Err(err) => HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() })),
                }
            },
            Ok(None) => HttpResponse::Unauthorized().json(json!({ "code": 401, "success": false, "error": "Could not authenticate request." })),
            Err(err) => HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({ "code": 401, "success": false, "error": "Could not authenticate request." }))
    }
}

#[patch("/email")]
async fn update_email(client: web::Data<Client>, req: HttpRequest, params: web::Form<UpdateEmailParams>) -> HttpResponse {
    if !verified_csrf(&req) {
        return HttpResponse::Unauthorized().json(json!({ "status": 401, "success": false, "error": "Invalid CSRF Token!" }));
    }
    if let Some(token) = get_session_token(&req) {
        let collection: Collection<Accounts> = client.database("ouja_skins").collection("accounts");
        match collection.find_one(doc! { "session": token }, None).await {
            Ok(Some(account)) => {
                if params.email.len() > 256 {
                    return HttpResponse::Ok().json(json!({ "status": 200, "success": false, "error": "Email is too long!" }))
                }
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
async fn login(client: web::Data<Client>, req: HttpRequest, params: web::Form<LoginParams>) -> HttpResponse {
    if !verified_csrf(&req) {
        return HttpResponse::Unauthorized().json(json!({ "status": 401, "success": false, "error": "Invalid CSRF Token!" }));
    }
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
