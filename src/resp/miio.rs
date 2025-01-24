use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultData<T> {
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiIODevices {
    pub list: Vec<MiIODevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiIODevice {
    pub did: String,
    pub token: String,
    pub longitude: String,
    pub latitude: String,
    pub name: String,
    pub pid: String,
    pub localip: String,
    pub mac: String,
    pub ssid: String,
    pub bssid: String,
    pub parent_id: String,
    pub parent_model: String,
    pub show_mode: i32,
    pub model: String,
    #[serde(rename = "adminFlag")]
    pub admin_flag: i32,
    #[serde(rename = "shareFlag")]
    pub share_flag: i32,
    #[serde(rename = "permitLevel")]
    pub permit_level: i32,
    #[serde(rename = "isOnline")]
    pub is_online: bool,
    pub desc: String,
    pub extra: Extra,
    pub uid: i64,
    pub pd_id: i64,
    pub password: String,
    pub p2p_id: String,
    pub rssi: i32,
    pub family_id: i64,
    pub reset_flag: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extra {
    #[serde(rename = "isSetPincode")]
    pub is_set_pincode: i32,
    #[serde(rename = "pincodeType")]
    pub pincode_type: i32,
    pub fw_version: String,
    #[serde(rename = "needVerifyCode")]
    pub need_verify_code: i32,
    #[serde(rename = "isPasswordEncrypt")]
    pub is_password_encrypt: i32,
}
