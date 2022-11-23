use actix_web::HttpRequest;

use crate::magic_crypt::decrypt;

pub fn get_skins_path() -> String {
    return dotenvy::var("SKINS_PATH").unwrap();
}

pub fn get_session_token<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    return req.headers().get("x-session")?.to_str().ok();
}

pub fn get_csrf_token<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    return req.headers().get("x-csrf")?.to_str().ok();
}

pub fn verified_csrf<'a>(req: &'a HttpRequest) -> bool {
    if let Some(token) = get_csrf_token(&req) {
        return decrypt(token).len() > 0;
    } else {
        return false;
    }
}
