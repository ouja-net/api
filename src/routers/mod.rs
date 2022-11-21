use actix_web::web;

mod account;
mod user;

pub fn v1(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("v1").configure(v1_config));
}

pub fn v1_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("user").service(user::index));
    cfg.service(
        web::scope("account")
            .service(account::me)
            .service(account::login)
            .service(account::register)
            .service(account::update_email),
    );
}
