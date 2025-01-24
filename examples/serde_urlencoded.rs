use serde::Serialize;

#[derive(Serialize)]
struct GenericFormData<T>
where
    T: Serialize,
{
    key1: String,
    key2: T,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = GenericFormData {
        key1: "value1".to_string(),
        key2: 42, // 泛型参数为整数
    };

    // 将结构体转换为表单格式字符串
    let form_encoded = serde_urlencoded::to_string(&data)?;
    println!("Form encoded: {}", form_encoded); // 输出: key1=value1&key2=42

    let data2 = GenericFormData {
        key1: "value1".to_string(),
        key2: serde_json::json!({
            "getVirtualModel": false,
            "getHuamiDevices": 0,
        }), // 泛型参数为整数
    };

    // 将结构体转换为表单格式字符串
    let form_encoded2 = serde_urlencoded::to_string(&data2)?;
    println!("Form encoded: {}", form_encoded2);

    Ok(())
}
