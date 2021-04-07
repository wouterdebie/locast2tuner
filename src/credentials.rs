use crate::config::Config;
use chrono::{DateTime, Utc};
use futures::lock::Mutex;
use log::info;
use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

static LOGIN_URL: &str = "https://api.locastnet.org/api/user/login";
static USER_URL: &str = "https://api.locastnet.org/api/user/me";
static TOKEN_LIFETIME: i64 = 3600;

// Struct that holds the locast token and is able to login to the locast service
#[derive(Debug)]
pub struct LocastCredentials {
    config: Arc<Config>,
    token: Arc<Mutex<String>>,
    last_login: Arc<Mutex<DateTime<Utc>>>,
}

impl LocastCredentials {
    // Construct a new object
    pub async fn new(config: Arc<Config>) -> LocastCredentials {
        let token = login(&(config.username), &(config.password)).await;
        validate_user(&token).await;
        LocastCredentials {
            config,
            token: Arc::new(Mutex::new(token)),
            last_login: Arc::new(Mutex::new(Utc::now())),
        }
    }

    // Retrieve the locast token (used for subsequent authenticated  requests).
    // This will first validate the token.
    pub async fn token(&self) -> String {
        self.validate_token().await;
        self.token.lock().await.to_owned()
    }

    // Validate the login token by comparing it to `TOKEN_LIFETIME`. If it has expired,
    // a new login attempt will be made.
    pub async fn validate_token(&self) {
        let mut last_login = self.last_login.lock().await;
        if (Utc::now() - *last_login).num_seconds() < TOKEN_LIFETIME {
            return;
        }
        info!("Login token expired: {:?}", self.last_login);

        // Lock the token and try to login. Then set the new token and reset last_login.
        let mut token = self.token.lock().await;
        *token = login(&(self.config.username), &(self.config.password)).await;
        *last_login = Utc::now();
    }
}

// Log in to locast.org
async fn login<'a>(username: &str, password: &str) -> String {
    info!("Logging in with {}", username);
    let credentials = json!({
        "username": username,
        "password": password
    });

    // Login to locast
    let resp = reqwest::Client::new()
        .post(LOGIN_URL)
        .json(&credentials)
        .headers(crate::utils::construct_headers())
        .send()
        .await
        .unwrap();

    if !resp.status().is_success() {
        panic!("Login failed");
    } else {
        info!("Login succeeded!");
    }

    resp.json::<HashMap<String, String>>().await.unwrap()["token"].clone()
}
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct UserInfo {
    didDonate: bool,
    donationExpire: i64,
}

// Validate the locast user and make sure the user has donated and the donation didn't expire.
// If invalid, panic.
async fn validate_user(token: &str) {
    let user_info: UserInfo = crate::utils::get(USER_URL, Some(token))
        .await
        .json()
        .await
        .unwrap();
    let now = Utc::now().timestamp();
    if user_info.didDonate && now > user_info.donationExpire / 1000 {
        panic!("Donation expired!")
    } else if !user_info.didDonate {
        panic!("User didn't donate!")
    }
}
