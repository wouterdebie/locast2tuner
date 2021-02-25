use crate::config::Config;
use chrono::{DateTime, Utc};
use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

static LOGIN_URL: &str = "https://api.locastnet.org/api/user/login";
static USER_URL: &str = "https://api.locastnet.org/api/user/me";
static TOKEN_LIFETIME: i64 = 3600;

#[derive(Debug)]
pub struct LocastCredentials {
    config: Arc<Config>,
    token: Arc<Mutex<String>>,
    last_login: Arc<Mutex<DateTime<Utc>>>,
}

impl LocastCredentials {
    pub fn new(config: Arc<Config>) -> LocastCredentials {
        let token = login(&(config.username), &(config.password));
        validate_user(&token);
        let last_login = Utc::now();
        LocastCredentials {
            config,
            token: Arc::new(Mutex::new(token)),
            last_login: Arc::new(Mutex::new(last_login)),
        }
    }

    pub fn token(&self) -> String {
        self.validate_token();
        self.token.lock().unwrap().to_owned()
    }

    pub fn validate_token(&self) {
        let mut last_login = self.last_login.lock().unwrap();
        if (Utc::now() - *last_login).num_seconds() < TOKEN_LIFETIME {
            return;
        }
        println!("Login token expired: {:?}", self.last_login);

        let mut token = self.token.lock().unwrap();
        *token = login(&(self.config.username), &(self.config.password));
        *last_login = Utc::now();
    }
}

fn login<'a>(username: &str, password: &str) -> String {
    println!("Logging in with {}", username);
    let credentials = json!({
        "username": username,
        "password": password
    });

    let resp = reqwest::blocking::Client::new()
        .post(LOGIN_URL)
        .json(&credentials)
        .headers(crate::utils::construct_headers())
        .send()
        .unwrap();

    if !resp.status().is_success() {
        panic!("Login failed");
    } else {
        println!("Login succeeded!");
    }

    resp.json::<HashMap<String, String>>().unwrap()["token"].clone()
}
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct UserInfo {
    didDonate: bool,
    donationExpire: i64,
}

fn validate_user(token: &str) {
    let user_info: UserInfo = crate::utils::get(USER_URL, Some(token)).json().unwrap();
    let now = Utc::now().timestamp();
    if user_info.didDonate && now > user_info.donationExpire / 1000 {
        panic!("Donation expired!")
    } else if !user_info.didDonate {
        panic!("User didn't donate!")
    }
}
