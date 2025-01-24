use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use log::debug;
use rand::Rng;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::{
    resp::{MiIODevices, Response, ResultData},
    Account, MIIO_SID,
};

pub struct MiIOService {
    account: Account,
    server: String,
}

impl MiIOService {
    pub fn new(account: Account, region: Option<&str>) -> Self {
        let r = region
            .map(|s| {
                if s == "cn" {
                    "".to_owned()
                } else {
                    format!("{s}.")
                }
            })
            .unwrap_or_default();

        let server = format!("https://{r}api.io.mi.com/app");
        // 检查添加PassportDeviceId cookie
        {
            let mut store = account.token.cookies.lock().unwrap();
            let request_url = Url::parse(&server).unwrap();
            let domain = request_url.domain().unwrap();
            if !store.contains(domain, "/", "PassportDeviceId") {
                let cookie = cookie::Cookie::build(("PassportDeviceId", &account.token.device_id))
                    .domain(domain)
                    .path("/")
                    .build();
                store.insert_raw(&cookie, &request_url).unwrap();
            }
        }

        Self { account, server }
    }

    async fn request<R, P>(&mut self, uri: &str, data: Option<P>) -> Result<Response<R>>
    where
        R: for<'de> Deserialize<'de>,
        P: Serialize + Clone,
    {
        debug!("MiIOService::request");
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            "iOS-14.4-6.0.103-iPhone12,3--D7744744F7AF32F0544445285880DD63E47D9BE9-8816080-84A3F44E137B71AE-iPhone".parse().unwrap(),
        );
        headers.insert(
            "x-xiaomi-protocal-flag-cli",
            "PROTOCAL-HTTP2".parse().unwrap(),
        );

        let ssecurity = self.account.get_sid(MIIO_SID).await?;
        let data = sign_data(uri, data, &ssecurity);

        let url = format!("{}{uri}", self.server);
        let res = self
            .account
            .request::<R, SignData>(MIIO_SID, &url, data, Some(headers), None)
            .await?;
        Ok(res)
    }

    pub async fn devices(
        &mut self,
        get_virtual_model: Option<bool>,
        get_huami_device: Option<bool>,
    ) -> Result<()> {
        debug!("MiIOService::devices");
        let resp: Response<ResultData<MiIODevices>> = self
            .request(
                "/home/device_list",
                Some(ParamDeviceList {
                    get_virtual_model: get_virtual_model.unwrap_or_default(),
                    get_huami_devices: get_huami_device.unwrap_or_default() as i32,
                }),
            )
            .await?;

        println!("{}", serde_json::to_string(&resp.data)?);
        Ok(())
    }
}

/// 签名相关的静态方法
fn sign_nonce(ssecurity: &str, nonce: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(STANDARD.decode(ssecurity).unwrap());
    hasher.update(STANDARD.decode(nonce).unwrap());
    STANDARD.encode(hasher.finalize())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParamDeviceList {
    #[serde(rename = "getVirtualModel")]
    get_virtual_model: bool,
    #[serde(rename = "getHuamiDevices")]
    get_huami_devices: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignData {
    #[serde(rename = "_nonce")]
    nonce: String,
    signature: String,
    data: String,
}

pub fn sign_data<T>(uri: &str, data: Option<T>, ssecurity: &str) -> Option<SignData>
where
    T: Serialize + Clone,
{
    let data = data
        .clone()
        .and_then(|value| serde_json::to_string(&value).ok())
        .unwrap_or_default();
    // let data_str = r#"{"getVirtualModel": false, "getHuamiDevices": 0}"#;
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..8).map(|_| rng.gen()).collect();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 60;

    let mut nonce_data = random_bytes;
    nonce_data.extend_from_slice(&timestamp.to_be_bytes()[4..]);

    let nonce = STANDARD.encode(&nonce_data);
    let snonce = sign_nonce(ssecurity, &nonce);

    let msg = format!("{}&{}&{}&data={}", uri, snonce, nonce, data);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&STANDARD.decode(&snonce).unwrap())
        .expect("HMAC can take key of any size");
    mac.update(msg.as_bytes());
    let result = mac.finalize();

    Some(SignData {
        nonce,
        data,
        signature: STANDARD.encode(result.into_bytes()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_data1() {
        let ssecurity = "pgnsv9VeDFb1YAi/75n8ew==";
        let data = serde_json::json!({
            "getVirtualModel": false,
            "getHuamiDevices": 0,
        });
        let data = serde_json::to_string(&data).unwrap();
        let random: [u8; 8] = [233, 73, 48, 166, 84, 185, 56, 189];
        let timestamp2 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 60;

        let timestamp: u64 = 28958944;
        println!("timestamp:{timestamp}--timestamp2:{timestamp2}");
        let big_byte_time = &timestamp.to_be_bytes()[4..];
        assert_eq!([1, 185, 224, 224], big_byte_time);

        let mut nonce_data = random.clone().to_vec();
        nonce_data.extend_from_slice(big_byte_time);

        let nonce = STANDARD.encode(&nonce_data);
        assert_eq!("6UkwplS5OL0BueDg", nonce);
        let snonce = sign_nonce(ssecurity, &nonce);
        assert_eq!("86GVzHJQkMjUqxsSphKtd+2c5x9WqhOBdVcUT8is89Q=", snonce);
        let msg = format!("/home/device_list&{}&{}&data={}", snonce, nonce, data);
        assert_eq!(
            r#"/home/device_list&86GVzHJQkMjUqxsSphKtd+2c5x9WqhOBdVcUT8is89Q=&6UkwplS5OL0BueDg&data={"getVirtualModel": false, "getHuamiDevices": 0}"#,
            msg
        );

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(&STANDARD.decode(&snonce).unwrap())
            .expect("HMAC can take key of any size");
        mac.update(msg.as_bytes());
        let result = mac.finalize();
        let signature = STANDARD.encode(result.into_bytes());
        assert_eq!("+4ElcsBEOqEeoBstLDr6VycknHEFTwRScdK7GATrfIs=", signature);
    }
}
