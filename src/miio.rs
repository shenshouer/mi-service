use std::{
    env,
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use log::debug;
use rand::Rng;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use tokio::{fs, sync::Mutex};
use url::Url;

use crate::{
    resp::{MiIODevice, MiIODevices, Response, ResultData},
    Account, MIIO_SID,
};

pub struct MiIOService {
    account: Arc<Mutex<Account>>,
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

        let account = Arc::new(Mutex::new(account));
        Self { account, server }
    }

    async fn request<R, P>(&self, uri: &str, data: P) -> Result<Response<R>>
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

        let mut account = self.account.lock().await;
        let ssecurity = account.get_sid(MIIO_SID).await?;
        let data = sign_data(uri, data, &ssecurity);

        let url = format!("{}{uri}", self.server);
        let res = account
            .request::<R, SignData>(MIIO_SID, &url, data, Some(headers), None)
            .await?;
        Ok(res)
    }

    pub async fn home_request(&self, did: &str, method: &str, params: Value) -> Result<Value> {
        debug!("MiIOService::home_request");
        let data = Some(json!({
            "id": 1,
            "method": method,
            "accessKey": "IOS00026747c5acafc2",
            "params": params,
        }));
        let resp = self.request(&format!("/home/rpc/{}", did), data).await?;
        Ok(resp.data)
    }

    pub async fn home_get_props(&self, did: &str, props: Vec<String>) -> Result<Value> {
        debug!("MiIOService::home_get_props");
        self.home_request(did, "get_prop", json!(props)).await
    }

    pub async fn home_set_props(&self, did: &str, props: Vec<(String, Value)>) -> Result<Vec<i32>> {
        debug!("MiIOService::home_set_props");
        let mut results = Vec::new();
        for (prop, value) in props {
            let result = self.home_set_prop(did, &prop, value).await?;
            results.push(result);
        }
        Ok(results)
    }

    pub async fn home_get_prop(&self, did: &str, prop: &str) -> Result<Value> {
        debug!("MiIOService::home_get_prop");
        let result = self.home_get_props(did, vec![prop.to_owned()]).await?;
        Ok(result[0].clone())
    }

    pub async fn home_set_prop(&self, did: &str, prop: &str, value: Value) -> Result<i32> {
        let value = match value {
            Value::Array(_) => value,
            _ => json!([value]),
        };

        let result = self
            .home_request(did, &format!("set_{}", prop), value)
            .await?;
        Ok(if result[0] == "ok" {
            0
        } else {
            result[0].as_i64().unwrap_or(-1) as i32
        })
    }

    // MIOT相关方法
    pub async fn miot_request(&self, cmd: &str, params: Value) -> Result<Value> {
        debug!("MiIOService::miot_request");
        let resp = self
            .request(&format!("/miotspec/{}", cmd), json!({"params": params}))
            .await?;
        Ok(resp.data)
    }

    pub async fn miot_get_props(
        &self,
        did: &str,
        iids: Vec<(i32, i32)>,
    ) -> Result<Vec<Option<Value>>> {
        debug!("MiIOService::miot_get_props");
        let params: Vec<Value> = iids
            .iter()
            .map(|(siid, piid)| {
                json!({
                    "did": did,
                    "siid": siid,
                    "piid": piid
                })
            })
            .collect();

        let result = self.miot_request("prop/get", json!(params)).await?;

        Ok(result
            .as_array()
            .unwrap()
            .iter()
            .map(|it| {
                if it["code"] == 0 {
                    Some(it["value"].clone())
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn miot_set_props(
        &self,
        did: &str,
        props: Vec<(i32, i32, Value)>,
    ) -> Result<Vec<i32>> {
        debug!("MiIOService::miot_set_props");
        let params: Vec<Value> = props
            .iter()
            .map(|(siid, piid, value)| {
                json!({
                    "did": did,
                    "siid": siid,
                    "piid": piid,
                    "value": value
                })
            })
            .collect();

        let result = self.miot_request("prop/set", json!(params)).await?;

        Ok(result
            .as_array()
            .unwrap()
            .iter()
            .map(|it| it["code"].as_i64().unwrap_or(-1) as i32)
            .collect())
    }

    pub async fn miot_get_prop(&self, did: &str, iid: (i32, i32)) -> Result<Option<Value>> {
        debug!("MiIOService::miot_get_prop");
        let props = self.miot_get_props(did, vec![iid]).await?;
        Ok(props[0].clone())
    }

    pub async fn miot_set_prop(&self, did: &str, iid: (i32, i32), value: Value) -> Result<i32> {
        debug!("MiIOService::miot_set_prop");
        let results = self
            .miot_set_props(did, vec![(iid.0, iid.1, value)])
            .await?;
        Ok(results[0])
    }

    pub async fn miot_action(
        &self,
        did: &str,
        iid: (i32, i32),
        args: Option<Vec<Value>>,
    ) -> Result<i32> {
        debug!("MiIOService::miot_action");
        let params = json!({
            "did": did,
            "siid": iid.0,
            "aiid": iid.1,
            "in": args.unwrap_or_default()
        });

        let result = self.miot_request("action", params).await?;
        Ok(result["code"].as_i64().unwrap_or(-1) as i32)
    }

    pub async fn devices(
        &self,
        name: Option<&str>,
        get_virtual_model: Option<bool>,
        get_huami_device: Option<bool>,
    ) -> Result<Vec<MiIODevice>> {
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
        let list = resp.data.result.list;
        match name {
            Some(n) => Ok(list.into_iter().filter(|it| it.name.contains(n)).collect()),
            None => Ok(list),
        }
    }

    // MIOT规格解析
    pub async fn miot_spec(
        &self,
        type_filter: Option<&str>,
        format: Option<&str>,
    ) -> Result<Value> {
        if !type_filter.map(|t| t.starts_with("urn")).unwrap_or(false) {
            let cache_path = if let Ok(path) = env::var("MIOT_SPEC_PATH") {
                let temp_dir = Path::new(&path);
                temp_dir.join("miservice_miot_specs.json")
            } else {
                tempdir()?.path().join("miservice_miot_specs.json")
            };

            let specs = if cache_path.exists() {
                let content = fs::read_to_string(&cache_path).await?;
                serde_json::from_str(&content)?
            } else {
                let client = reqwest::Client::new();
                let resp = client
                    .get("http://miot-spec.org/miot-spec-v2/instances?status=all")
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;

                let instances = resp["instances"].as_array().unwrap();
                let mut specs = json!({});
                for instance in instances {
                    specs[instance["model"].as_str().unwrap()] = instance["type"].clone();
                }

                fs::write(&cache_path, serde_json::to_string(&specs)?).await?;
                specs
            };

            // 根据type_filter过滤规格
            let filtered_specs = if let Some(filter) = type_filter {
                let mut result = json!({});
                for (model, spec_type) in specs.as_object().unwrap() {
                    if model == filter {
                        result[model] = spec_type.clone();
                        break;
                    } else if model.contains(filter) {
                        result[model] = spec_type.clone();
                    }
                }
                result
            } else {
                specs
            };

            if filtered_specs.as_object().unwrap().len() != 1 {
                return Ok(filtered_specs);
            }

            let spec_type = filtered_specs.as_object().unwrap().values().next().unwrap();
            return self
                .fetch_spec_details(spec_type.as_str().unwrap(), format)
                .await;
        }

        self.fetch_spec_details(type_filter.unwrap(), format).await
    }

    /// 获取规格详情
    /// TODO: 实现格式化输出
    async fn fetch_spec_details(&self, spec_type: &str, _format: Option<&str>) -> Result<Value> {
        let url = format!(
            "http://miot-spec.org/miot-spec-v2/instance?type={}",
            spec_type
        );
        let client = reqwest::Client::new();
        let result = client.get(&url).send().await?.json::<Value>().await?;

        // 这里需要实现格式化输出的逻辑
        // 根据format参数(python或其他)生成相应格式的输出
        // 具体实现比较复杂，需要根据实际需求来完成
        // match format {
        //     Some(f) if f == "json" => {
        //     }
        //     _ => Ok(result),
        // }
        Ok(result)
    }

    // MIOT解码
    pub fn miot_decode(ssecurity: &str, nonce: &str, data: &str, gzip: bool) -> Result<Value> {
        use rc4::{KeyInit, Rc4, StreamCipher};
        use std::io::Read;

        let key = STANDARD.decode(sign_nonce(ssecurity, nonce))?;
        let mut cipher = Rc4::<rc4::consts::U256>::new_from_slice(key.as_slice())?;

        // Skip first 1024 bytes
        let mut skip = vec![0u8; 1024];
        cipher.apply_keystream(&mut skip);

        let mut decoded = STANDARD.decode(data)?;
        cipher.apply_keystream(&mut decoded);

        if gzip {
            use flate2::read::GzDecoder;
            let mut decoder = GzDecoder::new(&decoded[..]);
            let mut decompressed = Vec::new();
            if decoder.read_to_end(&mut decompressed).is_ok() {
                decoded = decompressed;
            }
        }

        Ok(serde_json::from_slice(&decoded)?)
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

pub fn sign_data<T>(uri: &str, data: T, ssecurity: &str) -> Option<SignData>
where
    T: Serialize + Clone,
{
    let data = serde_json::to_string(&data).ok()?;
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
