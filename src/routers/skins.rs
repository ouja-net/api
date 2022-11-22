use std::{fs, io::Write, str::from_utf8};

use actix_multipart::{Multipart};
use actix_web::{put, web, HttpRequest, HttpResponse};
use bson::doc;
use futures_util::stream::StreamExt as _;
use mongodb::{bson::DateTime, Client, Collection};
use serde_json::json;
use uuid::Uuid;

use crate::{
    magic_crypt::encrypt,
    models::{Accounts, SkinCollection, SkinMeta},
    util::{get_session_token, get_skins_path},
};

#[put("/upload")]
pub async fn upload_skin(
    client: web::Data<Client>,
    mut payload: Multipart,
    req: HttpRequest,
) -> HttpResponse {
    if let Some(token) = get_session_token(&req) {
        let collection_accounts: Collection<Accounts> =
            client.database("ouja_skins").collection("accounts");
        let collection_skins: Collection<SkinCollection> =
            client.database("ouja_skins").collection("skins");

        match collection_accounts.find_one(doc! { "session": token }, None).await {
            Ok(Some(_account)) => {
                let mut file_size: usize = 0;
                let mut buffer: Vec<u8> = Vec::new();
                let mut file_name: String = "".to_string();
                let mut title: String = "".to_string();
                let mut description: String = "".to_string();

                while let Some(Ok(mut field)) = payload.next().await {
                    if field.name() == "title" {
                        while let Some(chunk) = field.next().await {
                            let data = chunk.unwrap();
                            title = from_utf8(&data.to_vec()).unwrap().to_string();
                        }
                    }
                    if field.name() == "description" {
                        while let Some(chunk) = field.next().await {
                            let data = chunk.unwrap();
                            description = from_utf8(&data.to_vec()).unwrap().to_string();
                        }
                    }
                    if field.name() == "skin" {
                        while let Some(chunk) = field.next().await {
                            let data = chunk.unwrap();
                            file_size += data.len();
                            buffer.append(&mut data.to_vec());
                        }
                        let content_dis = field.content_disposition();
                        let filename = content_dis.get_filename().ok_or("Unknown file name").unwrap().to_string().to_owned();
                        file_name = filename;
                    }
                }

                if file_size == 0 || buffer.len() == 0 {
                    return HttpResponse::NotFound().json(json!({ "status": 400, "success": false, "error": "Could not find skin file." }))
                }

                if file_size > 5000 {
                    return HttpResponse::PayloadTooLarge().json(json!({ "status": 413, "success": false, "error": "Skin file is too large. It must be less than 5KB!" }))
                }

                if title.len() > 16 {
                    return HttpResponse::Forbidden().json(json!({ "status": 403, "success": false, "error": "Title cannot be larger than 16 characters!" }))
                }

                if description.len() > 256 {
                    return HttpResponse::Forbidden().json(json!({ "status": 403, "success": false, "error": "Description cannot be larger than 256 characters!" }))
                }

                let content_type = tree_magic::from_u8(&buffer);
                let meta = match &content_type[..] {
                    "image/jpeg" | "image/png" => {
                        if let Ok(imagesize::ImageSize { width, height }) = imagesize::blob_size(&buffer) {
                            if height != 32 && height != 64 || width != 64 {
                                return HttpResponse::Ok().json(json!({ "status": 400, "success": false, "error": "Skin must be 64x64 or 64x32" }));
                            }
                            SkinMeta::Image {
                                width,
                                height,
                                content_type: content_type.to_string(),
                            }
                        } else {
                            SkinMeta::Image {
                                width: 0,
                                height: 0,
                                content_type: "".to_string(),
                            }
                        }
                    },
                    &_ => {
                        HttpResponse::Ok().json(
                            json!({ "status": 500, "success": false, "error": "Skin must be a png or jpeg!" }),
                        );
                        SkinMeta::Image {
                            width: 0,
                            height: 0,
                            content_type: "".to_string(),
                        }
                    }
                };

                // I genuinely have no idea if this hash system will work properly.
                // By the looks of it, it *should*. But if anyone knows a better way, make a pull request please.
                let utf8_string = String::from_utf8_lossy(&buffer);
                let mut hash: String = encrypt(&utf8_string);
                hash = (&hash[&hash.len() - 48..]).to_string();

                // Checking if the skin already exists by searching the image hash.
                match collection_skins.find_one(doc! { "hash": &hash }, None).await {
                    Err(err) => {
                        HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() }))
                    }
                    Ok(Some(_skin)) => {
                        HttpResponse::Forbidden().json(json!({ "status": 403, "success": false, "error": "Skin file already exists!" }))
                    },
                    Ok(None) => {
                        // Now checking if the title already exists
                        match collection_skins.find_one(doc! { "title": { "$regex": "^".to_owned() + &title.to_lowercase() + "$", "$options": "i" } }, None).await {
                            Err(err) => {
                                HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() }))
                            },
                            Ok(Some(_skin)) => {
                                HttpResponse::Forbidden().json(json!({ "status": 401, "success": false, "error": "Title already exists!" }))
                            }
                            Ok(None) => {
                                let skin = SkinCollection {
                                    date: DateTime::now(),
                                    id: Uuid::new_v4().to_string(),
                                    hash,
                                    filename: file_name,
                                    size: file_size.clone(),
                                    title,
                                    description,
                                    metadata: meta,
                                };
                
                                let skins_path = get_skins_path();
                                let mut new_file = fs::File::create(format!("{}/{}.png", skins_path, &skin.id)).expect("There was an error creating a skin file!");
        
                                match new_file.write_all(&buffer) {
                                    Ok(()) => {
                                        match collection_skins.insert_one(&skin, None).await {
                                            Ok(_result) => {
                                                HttpResponse::Ok()
                                                    .json(json!({ "status": 200, "success": true, "skin": &skin.id }))
                                            }
                                            Err(err) => HttpResponse::InternalServerError().json(
                                                json!({ "status": 500, "success": false, "error": err.to_string() }),
                                            ),
                                        }
                                    }
                                    Err(err) => {
                                        println!("{}", err.to_string());
                                        HttpResponse::InternalServerError()
                                            .json(json!({ "status": 500, "success": false, "error": err.to_string() }))
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Ok(None) => {
                HttpResponse::Unauthorized().json(json!({ "status": 401, "success": false, "error": "Could not authenticate request." }))
            },
            Err(err) => {
                HttpResponse::InternalServerError().json(json!({ "status": 500, "success": false, "error": err.to_string() }))
            }
        }
    } else {
        HttpResponse::Unauthorized().json(
            json!({ "status": 401, "success": false, "error": "Could not authenticate request." }),
        )
    }
}
