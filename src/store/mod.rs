use std::{collections::HashMap, path::Path, sync::Arc};

use log::{debug, error};
use reqwest_cookie_store::CookieStoreMutex;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::utils::get_random;

mod serde_cookies;

// Token存储结构体
#[derive(Clone)]
pub struct TokenStore {
    pub path: String,
    pub token: Token,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Token {
    pub device_id: String,
    pub user_id: String,
    pub pass_token: String,
    // pub sid: HashMap<String, (String, String)>, // key: sid, value (ssecurity, service_token)
    pub sid: HashMap<String, String>, // key: sid, value (ssecurity, service_token)
    #[serde(with = "serde_cookies")]
    pub cookies: Arc<CookieStoreMutex>,
}

impl Token {
    pub fn clean(&mut self) {
        self.user_id = String::new();
        self.pass_token = String::new();
        self.sid = HashMap::new();
        let mut store = self.cookies.lock().unwrap();
        store.clear();
    }
}

impl TokenStore {
    pub async fn new(path: String) -> Self {
        let mut token = if Path::new(&path).exists() {
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => serde_json::from_str::<Token>(&content).unwrap(),
                Err(e) => {
                    panic!("Load token from {path} failed: {e}");
                }
            }
        } else {
            Token::default()
        };

        if token.device_id.is_empty() {
            token.device_id = get_random(16).to_uppercase();
        }

        {
            let mut store = token.cookies.lock().unwrap();
            let request_url = Url::parse("https://account.xiaomi.com").unwrap();
            if !store.contains("account.xiaomi.com", "/", "sdkVersion") {
                let cookie = cookie::Cookie::build(("sdkVersion", "3.9"))
                    .domain("account.xiaomi.com")
                    .path("/")
                    .build();
                store.insert_raw(&cookie, &request_url).unwrap();
            }

            if !store.contains("account.xiaomi.com", "/", "deviceId") {
                let cookie = cookie::Cookie::build(("deviceId", &token.device_id))
                    .domain("account.xiaomi.com")
                    .path("/")
                    .build();
                store.insert_raw(&cookie, &request_url).unwrap();
            }
        }

        Self { path, token }
    }

    pub async fn save(&self) {
        debug!("Token::save");
        if let Ok(content) = serde_json::to_string_pretty(&self.token) {
            let mut file = tokio::fs::File::create(&self.path)
                .await
                .expect("Open token file failed");

            if let Err(er) = file.write(content.as_bytes()).await {
                error!("Save token to {} failed: {er}", self.path);
            }
        }
    }

    pub async fn clean(&mut self) {
        if Path::new(&self.path).exists() {
            let _ = tokio::fs::remove_file(&self.path).await;
        }

        self.token.clean();
    }
}
