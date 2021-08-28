use crate::config::Config;
use chrono::{DateTime, Utc};
use futures::lock::Mutex;
use log::{error, info};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

static LOGIN_URL: &str =
    "https://api.locastnet.org/api/user/login?client_id=i7XBbuDLYwN%2Bd%2Bcf84YQCYK-NyR8QjBGyFV-g5zCRFY%3D";
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
        validate_user(&config, &token).await;
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
    debug!("Logging in with {}", username);
    let credentials = json!({
        "username": username,
        "password": password
    });

    let resp = match crate::utils::post(LOGIN_URL, credentials, 10000).await {
        Ok(r) if r.status() == StatusCode::OK => {
            info!("Login succeeded!");
            r
        }
        Ok(r) if r.status() == StatusCode::NOT_FOUND => {
            error!("Login failed! Incorrect credentials.");
            panic!("Login failed, status code: {}", r.status())
        }
        Ok(r) => {
            error!("Login failed, status code: {}", r.status());
            panic!("Login failed, status code: {}", r.status())
        }
        Err(e) => {
            error!("Login failed: {}", e);
            panic!("Login failed: {}", e)
        }
    };

    resp.json::<HashMap<String, String>>().await.unwrap()["token"].clone()
}
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct UserInfo {
    didDonate: bool,
    donationExpire: Option<i64>,
}

// Validate the locast user and make sure the user has donated and the donation didn't expire.
// If invalid, panic.
async fn validate_user(config: &Arc<Config>, token: &str) {
    let response = crate::utils::get(USER_URL, Some(token), 100).await;
    let text = response.unwrap().text().await.unwrap();
    let user_info: Result<UserInfo, serde_json::Error> = serde_json::from_str(&text);

    let now = Utc::now().timestamp();
    match user_info {
        Err(e) => {
            error!("Invalid response body: {}", text);
            panic!("Error while validating user: {}", e)
        }
        Ok(u) => {
            if config.disable_donation_check {
                warn!("Donation check disabled! Only use this with a valid Locast Cares account!")
            } else if !u.didDonate {
                panic!("User didn't donate! Make sure you have an active donation at locast.org!")
            } else if now > u.donationExpire.unwrap() / 1000 {
                panic!("Donation expired! Make sure you have an active donation at locast.org!")
            }
        }
    }
}
