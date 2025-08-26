use std::{sync::{Arc, Mutex}, time::{self, Duration, Instant}};

use actix_web::{cookie::Cookie, get, http::StatusCode, post, web, HttpRequest, HttpResponse, Responder};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::distr::SampleString;
use serde::Deserialize;

use crate::{write_config, AppState};

#[derive(Deserialize)]
struct AuthInfo {
    username: String,
    password: String
}

#[post("/auth/create")]
async fn create_account(app_state: web::Data<Mutex<AppState>>, info: web::Json<AuthInfo>) -> impl Responder {
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
                Err(_) => return HttpResponse::InternalServerError().body("Password hash couldn't be generated")
            }
        },
        Err(_) => {
            return HttpResponse::InternalServerError().body("Password salt couldn't be generated");
        }
    }
    
    if write_config(&new_conf).is_err() {
        return HttpResponse::InternalServerError().body("Couldn't save your login."); 
    }

    data.config = new_conf;
    

    match data.tokens.lock() {
        Ok(mut tokens) => {
            let mut exp = Instant::now();
            exp += Duration::from_secs(60 * 60 * 24 * 31); // Now + 31 days
            

            let token = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 32);
            tokens.insert(token.clone(), exp);

            let cookie = Cookie::new("Authorization", token);
            let mut response = HttpResponse::Ok().body("OK");
            match response.add_cookie(&cookie) {
                Ok(_) => return response,
                Err(_) => return HttpResponse::InternalServerError().body("Account created without a token. Try to log in.")
            }
        },
        Err(_) => return HttpResponse::InternalServerError().body("Account created without a token. Try to log in.")
    }



}

#[get("/auth/check")]
async fn get_check_token(app_state: web::Data<Mutex<AppState>>, req: HttpRequest) -> impl Responder {
    let data = app_state.lock().unwrap();
        match req.cookie("Authorization") {
            Some(cookie) => {
                match data.tokens.lock() {
                    Ok(tokens) => {
                        if tokens.contains_key(cookie.value()) {
                            return HttpResponse::Ok().body("OK")
                        } else {
                            return HttpResponse::Unauthorized().body("Invalid Authorization cookie")
                        }
                    },
                    Err(_) => {
                        return HttpResponse::InternalServerError().body("Internal Server Error")
                    }
                }
            },
            None => {
                return HttpResponse::Unauthorized().body("Missing Authorization cookie")
            }
        }
}