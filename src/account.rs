use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine};
use log::{debug, error};
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE, USER_AGENT},
    Client, StatusCode,
};
use reqwest_cookie_store::CookieStoreMutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use url::Url;

use crate::{
    errors::Error,
    resp::Response,
    store::{Token, TokenStore},
};

// Account主结构体
pub struct Account {
    store: Arc<CookieStoreMutex>,
    client: Client,
    username: String,
    password: String,
    token_store: TokenStore,
    pub token: Token,
}

impl Account {
    pub fn new(username: String, password: String, token_store: TokenStore) -> Self {
        let token = token_store.token.clone();
        let store = token.cookies.clone();
        let client = reqwest::Client::builder()
            .cookie_provider(store.clone())
            .build()
            .expect("Build http client failed");

        Self {
            store,
            client,
            username,
            password,
            token_store,
            token,
        }
    }

    // 登录方法
    pub async fn login(&mut self, sid: &str) -> bool {
        debug!("Account::login sid: {}", sid);

        let resp = match self
            .service_login(&format!("serviceLogin?sid={}&_json=true", sid), None)
            .await
        {
            Ok(resp) => {
                if resp["code"] != 0 {
                    let auth_param = json!({
                        "_json": "true",
                        "sid": resp["sid"],
                        "qs": resp["qs"],
                        "_sign": resp["_sign"],
                        "callback": resp["callback"],
                        "user": self.username,
                        "hash": format!("{:X}", md5::compute(self.password.as_bytes())),
                    });
                    match self
                        .service_login("serviceLoginAuth2", Some(auth_param))
                        .await
                    {
                        Ok(resp) => resp,
                        Err(e) => {
                            error!("serviceLoginAuth2 failed: {:?}", e);
                            return false;
                        }
                    }
                } else {
                    resp
                }
            }
            Err(e) => {
                error!("serviceLogin failed: {:?}", e);
                return false;
            }
        };

        // 处理登录成功
        self.token.user_id = resp["userId"].to_string();
        self.token.pass_token = resp["passToken"].as_str().unwrap_or_default().to_owned();

        let nonce = resp["nonce"].to_string();
        let ssecurity = resp["ssecurity"].as_str().unwrap_or_default();
        let location = resp["location"].as_str().unwrap_or_default();
        // 获取安全令牌
        match self
            .security_token_service(location, &nonce, ssecurity)
            .await
        {
            Ok(_) => {
                self.token.sid.insert(sid.to_owned(), ssecurity.to_owned());
                self.token_store.token = self.token.clone();
                self.token_store.save().await;
                true
            }
            Err(e) => {
                error!("Failed to get service token: {:?}", e);
                false
            }
        }
    }

    // 服务登录请求
    async fn service_login(&self, uri: &str, data: Option<Value>) -> Result<Value, Error> {
        let url = format!("https://account.xiaomi.com/pass/{}", uri);
        debug!("Account::service_login: url:{url} data: {data:?}");

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            USER_AGENT,
            "APP/com.xiaomi.mihome APPV/6.0.103 iosPassportSDK/3.9.0 iOS/14.4 miHSTS"
                .parse()
                .unwrap(),
        );

        let mut builder = self
            .client
            .request(
                if data.is_some() {
                    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
                    reqwest::Method::POST
                } else {
                    reqwest::Method::GET
                },
                &url,
            )
            .headers(headers);

        if let Some(data) = data {
            builder = builder.form(&data);
        }

        let response = builder
            .send()
            .await
            .map_err(|e| Error::ServiceLogin(format!("request url {url} failed: {e}")))?;
        let status_code = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| Error::ServiceLogin(format!("get text from {url} response error: {e}")))?;
        if status_code != StatusCode::OK {
            return Err(Error::ServiceLogin(format!(
                "request url {url} failed, message:{text}, status code: {status_code}",
            )));
        }
        let json_str = &text[11..];

        serde_json::from_str(json_str)
            .map_err(|e| Error::ServiceLogin(format!("serde text {json_str} to json error: {e}")))
    }

    // 安全令牌服务
    async fn security_token_service(
        &self,
        location: &str,
        nonce: &str,
        ssecurity: &str,
    ) -> Result<(), Error> {
        debug!("Account::security_token_service ");
        let nsec = format!("nonce={}&{}", nonce, ssecurity);
        let mut hasher = Sha1::new();
        hasher.update(nsec.as_bytes());
        let client_sign = STANDARD.encode(hasher.finalize());

        let url = format!(
            "{location}&clientSign={}",
            urlencoding::encode(&client_sign)
        );

        let u = Url::parse(&url)
            .map_err(|e| Error::SecurityTokenService(format!("parse url {url} failed: {e}")))?;

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                Error::SecurityTokenService(format!("request url {url} failed: {e}"))
            })?;

        let status_code = response.status();
        if status_code != StatusCode::OK {
            let text = response.text().await.map_err(|e| {
                Error::SecurityTokenService(format!("get text from {url} response error: {e}"))
            })?;
            return Err(Error::SecurityTokenService(format!(
                "request url {url} failed, message:{text}, status code: {status_code}",
            )));
        }

        if u.domain() == Some("sts.api.io.mi.com") {
            self.recreate_cookie_for_domain();
        }
        Ok(())
    }

    // Mi请求方法
    pub async fn request<R, P>(
        &mut self,
        sid: &str,
        url: &str,
        data: Option<P>,
        headers: Option<HeaderMap>,
        relogin: Option<bool>,
    ) -> Result<Response<R>, Error>
    where
        R: for<'de> Deserialize<'de>,
        P: Serialize + Clone,
    {
        debug!(
            "account::request {headers:?} url:{url} data:{:?}",
            serde_json::to_string(&data)
        );

        if self.token.sid.contains_key(sid) || self.login(sid).await {
            let method = if data.is_some() {
                reqwest::Method::POST
            } else {
                reqwest::Method::GET
            };

            let mut builder = self.client.request(method, url);

            if let Some(headers) = &headers {
                builder = builder.headers(headers.clone());
            }

            if let Some(data) = data.clone() {
                builder = builder.form(&data);
            }

            let response = builder
                .send()
                .await
                .map_err(|e| Error::Request(format!("request url {url} failed: {e}")))?;

            match response.status() {
                StatusCode::OK => {
                    let resp: Response<R> = response.json().await.map_err(|e| {
                        Error::Request(format!("serde response from url {url} failed: {e}"))
                    })?;

                    match resp.code {
                        0 => Ok(resp),
                        _ if resp.message.to_lowercase().contains("auth")
                            && relogin.unwrap_or(true) =>
                        {
                            self.token.clean();
                            Box::pin(self.request(sid, url, data, headers, relogin)).await
                        }
                        _ => Err(Error::Request(format!(
                            "request url {url} API error: {}",
                            resp.message
                        ))),
                    }
                }
                status_code => {
                    let text = response.text().await;
                    Err(Error::Request(format!(
                        "request url {url} HTTP error: {status_code} {text:?}"
                    )))
                }
            }
        } else {
            Err(Error::Request("Login failed".to_owned()))
        }
    }

    pub async fn get_sid(&mut self, sid: &str) -> Result<String, Error> {
        if self.token.sid.contains_key(sid) || self.login(sid).await {
            if let Some(ssecurity) = self.token.sid.get(sid) {
                return Ok(ssecurity.to_owned());
            }
        }
        Err(Error::Request("get sid".to_owned()))
    }

    /// 重新为指定域名创建cookie
    /// 因为MiIOtoken获取的是sts.api.io.mi.com域名下的，而实际请求是api.io.mi.com域名
    /// 不能自动将cookie转移到api.io.mi.com域名下，所以需要手动创建
    fn recreate_cookie_for_domain(&self) {
        let mut store = self.store.lock().unwrap();
        let api_url = Url::parse("https://api.io.mi.com").unwrap();
        let raw_cookies = store
            .iter_any()
            .filter_map(|c| {
                if c.domain() == Some("sts.api.io.mi.com") {
                    let cookie_str = c.to_string();
                    let mut raw_cookie = cookie::Cookie::parse(cookie_str).unwrap();
                    raw_cookie.set_domain("api.io.mi.com");
                    Some(raw_cookie)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for cookie in raw_cookies {
            if let Err(e) = store.insert_raw(&cookie, &api_url) {
                error!("Failed to insert raw cookie {cookie:?} to store: {:?}", e);
            }
        }
    }
}
