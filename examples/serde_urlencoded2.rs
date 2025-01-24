use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParamDeviceList {
    #[serde(rename = "getVirtualModel")]
    get_virtual_model: bool,
    #[serde(rename = "getHuamiDevices")]
    get_huami_devices: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignData<T>
where
    T: Serialize,
{
    #[serde(rename = "_nonce")]
    nonce: String,
    signature: String,
    data: Option<T>,
}

fn main() {
    let data = json!({
        "getVirtualModel": false,
        "getHuamiDevices": 0,
    });

    let form = serde_urlencoded::to_string(&data);
    println!("form:{:?}", form);

    let param = ParamDeviceList {
        get_virtual_model: false,
        get_huami_devices: 0,
    };

    let form2 = serde_urlencoded::to_string(&param);
    println!("form2:{:?}", form2);

    let data3 = SignData {
        nonce: "123456".to_string(),
        signature: "abcdef".to_string(),
        data: Some(param),
    };
    let json_str3 = serde_json::to_string(&data3).unwrap();
    println!("json_str3:{:?}", json_str3);
    let form3 = serde_urlencoded::to_string(&data3);
    println!("form3:{:?}", form3);
}
