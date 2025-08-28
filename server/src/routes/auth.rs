use std::{
    sync::{Mutex},
    time::{Duration, Instant},
};

use actix_web::{
    Error, HttpResponse, Responder,
    body::MessageBody,
    cookie::Cookie,
    dev::{ServiceRequest, ServiceResponse},
    error::{ErrorUnauthorized},
    get,
    middleware::Next,
    post, web,
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString},
};
use rand::distr::SampleString;
use serde::Deserialize;

use crate::{AppState, write_config};

#[derive(Deserialize)]
struct AuthInfo {
    username: String,
    password: String,
}

#[post("/auth/create")]
async fn create_account(
    app_state: web::Data<Mutex<AppState>>,
    info: web::Json<AuthInfo>,
) -> impl Responder {
    let mut data = app_state.lock().unwrap();
    if data.config.username != "" || data.config.pass_hash != "" {
        return HttpResponse::Forbidden().body("A user was already created for this instance");
    }

    let mut new_conf = data.config.clone();
    new_conf.username = info.username.clone();

    let argon = Argon2::default();
    let salt = &SaltString::from_b64(data.config.salt.as_str());

    match salt {
        Ok(salt_string) => {
            let pass_hash = argon.hash_password(&info.password.as_bytes(), salt_string);
            match pass_hash {
                Ok(pass_hash) => new_conf.pass_hash = pass_hash.to_string(),
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .body("Password hash couldn't be generated");
                }
            }
        }
        Err(_) => {
            return HttpResponse::InternalServerError().body("Password salt couldn't be generated");
        }
    }

    if write_config(&new_conf).is_err() {
        return HttpResponse::InternalServerError().body("Couldn't save your login.");
    }

    data.config = new_conf;

    let (token, exp) = generate_token();
    data.tokens.insert(token.clone(), exp);
    let mut cookie = Cookie::new("Authorization", token);
    cookie.set_path("/");
    let mut response = HttpResponse::Ok().body("OK");
    match response.add_cookie(&cookie) {
        Ok(_) => return response,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Account created without a token. Try to log in.");
        }
    }
}

#[post("/auth/login")]
async fn login(app_state: web::Data<Mutex<AppState>>, info: web::Json<AuthInfo>) -> impl Responder {
    let mut data = app_state.lock().unwrap();
    let hash = match PasswordHash::new(data.config.pass_hash.as_str()) {
        Ok(pw_hash) => pw_hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Invalid configuration file on server's side");
        }
    };
    match Argon2::default().verify_password(info.password.as_bytes(), &hash) {
        Ok(_) => {
            let (token, exp) = generate_token();
            data.tokens.insert(token.clone(), exp);
            let mut cookie = Cookie::new("Authorization", token);
            cookie.set_path("/");

            let mut response = HttpResponse::Ok().body("OK");
            match response.add_cookie(&cookie) {
                Ok(_) => return response,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .body("Couldn't add the token cookie... please try again");
                }
            }
        }
        Err(_) => return HttpResponse::Unauthorized().body("Invalid username or password"),
    }
}

#[get("/check")] // under /protected scope
async fn get_check_token() -> impl Responder {
    //This is behind the check_token_middleware
    return HttpResponse::Ok().body("OK");
}

pub async fn check_token_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let app_state = req
        .app_data::<web::Data<Mutex<AppState>>>()
        .unwrap()
        .clone();
    let data = app_state.lock().unwrap();
    match req.cookie("Authorization") {
        Some(cookie) => {
            if data.tokens.contains_key(cookie.value()) {
                return next.call(req).await;
            } else {
                return Err(ErrorUnauthorized("Invalid Authorization cookie"));
            }
        }
        None => return Err(ErrorUnauthorized("Missing Authorization cookie")),
    }
}

fn generate_token() -> (String, Instant) {
    let mut exp = Instant::now();
    exp += Duration::from_secs(60 * 60 * 24 * 31); // Now + 31 days
    let token = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 32);
    return (token, exp);
}