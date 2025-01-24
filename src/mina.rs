use std::sync::Arc;

use anyhow::Result;
use log::{debug, error};
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use crate::{
    account::Account,
    resp::{MiNaDevices, Response},
    utils::get_random,
    MINA_SID,
};

pub struct MiNaService {
    account: Arc<Mutex<Account>>,
}

impl MiNaService {
    pub fn new(account: Account) -> Self {
        let account = Arc::new(Mutex::new(account));
        Self { account }
    }

    async fn request<T>(
        &self,
        mut uri: String,
        mut data: Option<serde_json::Value>,
    ) -> Result<Response<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("MiNaService::request");
        let req_id = format!("app_ios_{}", get_random(30));
        if let Some(data) = data.as_mut() {
            data["requestId"] = serde_json::Value::String(req_id);
        } else {
            uri = format!("{uri}&requestId={req_id}");
        }
        let mut headers = HeaderMap::new();
        headers.
            insert(
                USER_AGENT,
                "MiHome/6.0.103 (com.xiaomi.mihome; build:6.0.103.1; iOS 14.4.0) Alamofire/6.0.103 MICO/iOSApp/appStore/6.0.103".parse().unwrap(),
            );

        let mut account = self.account.lock().await;
        let resp = account
            .request(
                MINA_SID,
                &format!("https://api2.mina.mi.com{uri}"),
                data,
                Some(headers),
                None,
            )
            .await?;

        Ok(resp)
    }

    pub async fn devices(&self, master: Option<usize>) -> Result<MiNaDevices> {
        debug!("MiNaService::devices");
        let result = self
            .request(
                format!(
                    "/admin/v2/device_list?master={}",
                    master.unwrap_or_default()
                ),
                None,
            )
            .await?;

        Ok(result.data)
    }

    async fn ubus_request(
        &self,
        device_id: &str,
        method: &str,
        path: &str,
        message: serde_json::Value,
    ) -> Result<bool> {
        debug!("MiNaService::ubus_request");
        let resp: Response<serde_json::Value> = self
            .request(
                "/remote/ubus".to_string(),
                Some(json!({
                    "deviceId": device_id,
                    "message": message,
                    "method": method,
                    "path": path,
                })),
            )
            .await?;
        Ok(resp.code == 0)
    }

    pub async fn text_to_speech(&self, device_id: &str, text: &str) -> Result<bool> {
        debug!("MiNaService::text_to_speech");
        self.ubus_request(
            device_id,
            "text_to_speech",
            "mibrain",
            json!({
                "text": text,
            }),
        )
        .await
    }

    pub async fn player_set_volume(&self, device_id: &str, volume: i32) -> Result<bool> {
        debug!("MiNaService::player_set_volume");
        self.ubus_request(
            device_id,
            "player_set_volume",
            "mediaplayer",
            json!({
                "volume": volume,
                "media": "app_ios",
            }),
        )
        .await
    }

    /// 发生消息 或 调整 设备音量
    pub async fn send_message(
        &self,
        devices: &[serde_json::Value], // 假设使用 serde_json::Value 来表示设备数组
        devno: i32,                    // -1/0/1...
        message: Option<String>,
        volume: Option<i32>,
    ) -> Result<bool> {
        let mut result = false;

        for (i, device) in devices.iter().enumerate() {
            // 检查设备条件：devno == -1 或 devno != i+1 或设备支持 yunduantts
            if devno == -1
                || devno != (i + 1) as i32
                || device["capabilities"]["yunduantts"]
                    .as_bool()
                    .unwrap_or(false)
            {
                // 记录调试信息
                debug!(
                    "Send to devno={} index={}: {}",
                    devno,
                    i,
                    message
                        .as_ref()
                        .unwrap_or(&volume.map(|v| v.to_string()).unwrap_or_default())
                );

                let device_id = device["deviceID"].as_str().unwrap_or_default();

                // 设置音量或将结果设为 true
                result = if let Some(vol) = volume {
                    self.player_set_volume(device_id, vol).await?
                } else {
                    true
                };

                // 如果前面成功且有消息，则发送语音
                if result {
                    if let Some(msg) = &message {
                        result = self.text_to_speech(device_id, msg).await?;
                    }
                }

                // 发送失败时记录错误
                if !result {
                    error!(
                        "Send failed: {}",
                        message
                            .as_ref()
                            .unwrap_or(&volume.map(|v| v.to_string()).unwrap_or_default())
                    );
                }

                // 如果不是发送给所有设备(-1)或发送失败，则退出循环
                if devno != -1 || !result {
                    break;
                }
            }
        }

        Ok(result)
    }
}
