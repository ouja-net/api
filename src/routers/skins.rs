use std::{fs, io::Write};

use actix_multipart::Multipart;
use actix_web::{put, web, HttpResponse};
use bson::doc;
use futures_util::stream::StreamExt as _;
use mongodb::{bson::DateTime, Client, Collection};
use serde_json::json;
use uuid::Uuid;

use crate::{
    config::get_skins_path,
    magic_crypt::encrypt,
    models::{SkinCollection, SkinMeta},
};

#[put("/upload")]
pub async fn upload_skin(client: web::Data<Client>, mut payload: Multipart) -> HttpResponse {
    let mut skin_id: String = "".to_string();
    while let Some(Ok(mut field)) = payload.next().await {
        if field.name() == "skin" {
            let mut file_size: usize = 0;
            let mut buffer: Vec<u8> = Vec::new();

            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                file_size += data.len();

                buffer.append(&mut data.to_vec());
            }

            if file_size > 5000 {
                return HttpResponse::Ok().json(json!({ "status": 400, "success": false, "error": "Skin file cannot be larger than 5KB!" }));
            }

            let real_content_type = tree_magic::from_u8(&buffer);
            let meta = match &real_content_type[..] {
                "image/jpeg" | "image/png" => {
                    if let Ok(imagesize::ImageSize { width, height }) =
                        imagesize::blob_size(&buffer)
                    {
                        if height != 32 && height != 64 || width != 64 {
                            return HttpResponse::Ok().json(json!({ "status": 400, "success": false, "error": "Skin must be 64x64 or 64x32" }));
                        }
                        SkinMeta::Image {
                            width,
                            height,
                            content_type: real_content_type.to_string(),
                        }
                    } else {
                        HttpResponse::Ok().json(json!({ "status": 400, "success": false, "error": "Skin file must be a png or jpeg." }));
                        SkinMeta::Image {
                            width: 0,
                            height: 0,
                            content_type: "".to_string(),
                        }
                    }
                }
                &_ => {
                    HttpResponse::Ok().json(
                        json!({ "status": 500, "success": false, "error": "Unknown file type!" }),
                    );
                    SkinMeta::Image {
                        width: 0,
                        height: 0,
                        content_type: "".to_string(),
                    }
                }
            };

            let content_type = field.content_disposition();
            let filename = content_type
                .get_filename()
                .ok_or("Unknown file name")
                .unwrap();

            // I genuinely have no idea if this hash system will work properly.
            // By the looks of it, it *should*. But if anyone knows a better way, make a pull request please.
            let utf8_string = String::from_utf8_lossy(&buffer);
            let mut hash: String = encrypt(&utf8_string);
            hash = (&hash[&hash.len() - 48..]).to_string();

            let collection: Collection<SkinCollection> =
                client.database("ouja_skins").collection("skins");

            match collection.find_one(doc! { "hash": &hash }, None).await {
                Err(err) => {
                    return HttpResponse::InternalServerError().json(
                        json!({ "status": 500, "success": false, "error": err.to_string() }),
                    );
                }
                Ok(Some(_skin)) => {
                    return HttpResponse::Forbidden().json(json!({ "status": 403, "success": false, "error": "Skin file already exists!" }));
                }
                Ok(None) => {
                    let skin = SkinCollection {
                        date: DateTime::now(),
                        id: Uuid::new_v4().to_string(),
                        hash,
                        filename: filename.clone().to_string(),
                        size: file_size.clone(),
                        metadata: meta,
                    };
                    let skins_path = get_skins_path();
                    let mut new_file = fs::File::create(format!("{}/{}.png", skins_path, &skin.id))
                        .expect("There was an error!");

                    match new_file.write_all(&buffer) {
                        Ok(()) => {
                            match collection.insert_one(&skin, None).await {
                                Ok(_result) => {
                                    skin_id = skin.id;
                                    HttpResponse::Ok()
                                        .json(json!({ "status": 200, "success": true, "skin": skin_id }))
                                }
                                Err(err) => HttpResponse::InternalServerError().json(
                                    json!({ "status": 500, "success": false, "error": err.to_string() }),
                                ),
                            };
                        }
                        Err(err) => {
                            println!("{}", err.to_string());
                            HttpResponse::InternalServerError()
                                .json(json!({ "status": 500, "success": false, "error": err.to_string() }));
                        }
                    }
                }
            }
        }
    }
    HttpResponse::Ok().json(json!({ "status": 200, "success": true, "skin": skin_id }))
}
