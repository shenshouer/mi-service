use std::sync::Arc;

use cookie_store::Cookie;
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S>(cookies: &Arc<CookieStoreMutex>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let store = cookies.lock().unwrap();
    let cookies = store.iter_any().collect::<Vec<_>>();
    let value: serde_json::Value = serde_json::to_value(&cookies).unwrap();
    value.serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<CookieStoreMutex>, D::Error>
where
    D: Deserializer<'de>,
{
    let cookies: Vec<Cookie<'static>> = Vec::deserialize(deserializer)?;

    let cookie_store = CookieStore::from_cookies(cookies.into_iter().map(Ok), true)
        .map_err(|e: cookie_store::Error| <D::Error as serde::de::Error>::custom(e.to_string()))?;

    Ok(Arc::new(CookieStoreMutex::new(cookie_store)))
}

// pub fn serialize<S>(cookies: &CookieStore, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let mut bytes: Vec<u8> = Vec::new();
//     cookies
//         .save(&mut bytes, serde_json::to_string)
//         .map_err(|e| <S::Error as serde::ser::Error>::custom(e.to_string()))?;

//     serializer.serialize_bytes(&bytes)
// }

// pub fn deserialize<'de, D>(deserializer: D) -> Result<CookieStore, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s = String::deserialize(deserializer)?;
//     let reader = Cursor::new(s.into_bytes());
//     CookieStore::load(reader, |cookie| serde_json::from_str(cookie))
//         .map_err(|e| <D::Error as serde::de::Error>::custom(e.to_string()))
// }
