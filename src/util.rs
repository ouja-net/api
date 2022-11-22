use actix_web::HttpRequest;

pub fn get_skins_path() -> String {
    return dotenvy::var("SKINS_PATH").unwrap();
}

pub fn get_session_token<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    return req.headers().get("x-session")?.to_str().ok();
}
