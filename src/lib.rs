use std::collections::HashSet;

const DOMAIN: &str = "https://api.coinex.com";
const WS_DOMAIN: &str = "wss://socket.coinex.com";

pub struct MasterAPI(APIType);
impl MasterAPI {
    pub fn new(auth: APIType) -> MasterAPI {
        MasterAPI(auth)
    }
}


pub enum APIType {
    ReadOnly(APIAuth),
    Withdraw(APIAuth),
    Trade(APIAuth),
    ALL(APIAuth)
}

impl APIType {
    pub fn get_auth(&self) -> &APIAuth {
        match self {
            APIType::ReadOnly(auth) => auth,
            APIType::Withdraw(auth) => auth,
            APIType::Trade(auth) => auth,
            APIType::ALL(auth) => auth
        }
    }
}

impl std::ops::Deref for MasterAPI {
    type Target = APIAuth;
    fn deref(&self) -> &APIAuth {
        &self.0.get_auth()
    }
}

pub struct APIAuth {
    key: String,
    _secret: String,
    ip_whitelist: Option<HashSet<String>>,
}

impl APIAuth {
    pub fn new(key: &str, secret: &str, ip_whitelist: Option<HashSet<String>> ) -> APIAuth {
        APIAuth {
            key: String::from(key),
            _secret: String::from(secret),
            ip_whitelist,
        }
    }
    pub fn is_whitelisted(&self, ip: &str) -> bool {
        match self.ip_whitelist {
            Some(ref whitelist) => whitelist.contains(ip),
            None => true
        }
    }
    pub fn get_key(&self) -> &str {
        &self.key
    }
    fn sign(&self, payload: SignPayload) -> Signature {
        match payload {
            SignPayload::HTTP { verb, method, body, timestamp } => {
                let body = match body {
                    Some(body) => serde_json::to_string(&body).unwrap(),
                    None => String::new()
                };
                let payload = format!("{verb}{method}{body}{timestamp}", verb=verb.as_str());
                println!("{:?}", payload);
                let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, self._secret.as_bytes());
                Signature(hex::encode(ring::hmac::sign(&key, payload.as_bytes())).into())
            },
            SignPayload::Websocket { timestamp } => {
                let payload = format!("{timestamp}");
                let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, self._secret.as_bytes());
                Signature(hex::encode(ring::hmac::sign(&key, payload.as_bytes())).into())
            } 
        }
    }
}

struct Signature(String); //new typing the signature so compiler can yell at me if i dont generate it right


pub enum SignPayload {
    HTTP {
        verb: HTTPVerb,
        method: String,
        body: Option<serde_json::Value>,
        timestamp: i64
    },
    Websocket {
        timestamp: i64
    }
}

 

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum Capability {
    Trade,
    Withdraw
}

pub enum HTTPVerb {
    GET,
    POST,
    DELETE
}

impl HTTPVerb {
    pub fn as_str(&self) -> &str {
        match *self {
            HTTPVerb::GET => "GET",
            HTTPVerb::POST => "POST",
            HTTPVerb::DELETE => "DELETE"
        }
    }
}


pub async fn get_sub_account_list(client: &reqwest::Client, auth: &MasterAPI, user: Option<&str>, frozen: Option<bool>, page: Option<i32>, limit: Option<i32>) {
    let mut method = format!("/v2/account/subs");
    let mut query_string = String::new();
    if let Some(user) = user {
        query_string.push_str(&format!("&sub_user_name={user}"));
    }
    if let Some(frozen) = frozen {
        query_string.push_str(&format!("&is_frozen={frozen}"));
    }
    if let Some(page) = page {
        query_string.push_str(&format!("&page={page}"));
    }
    if let Some(limit) = limit {
        query_string.push_str(&format!("&limit={limit}"));
    }
    if !query_string.is_empty() {
        method = format!("{method}?{query_string}");
    }
    let timestamp = chrono::Utc::now().timestamp_millis();
    let res = client.get(&format!("{DOMAIN}{method}"))
        .header("X-COINEX-KEY", auth.get_key())
        .header("X-COINEX-SIGN", auth.sign(SignPayload::HTTP { verb: HTTPVerb::GET, method, body: None, timestamp }).0)
        .header("X-COINEX-TIMESTAMP", timestamp)
        .send().await;
    println!("{:#?}", res.unwrap().text().await);
}

pub async fn get_sub_account_api_list(client: &reqwest::Client, auth: &MasterAPI, user: &str, page: Option<i32>, limit: Option<i32>) {
    let mut method = format!("/v2/account/subs/api");
    let mut query_string = format!("?sub_user_name={user}");
    if let Some(page) = page {
        query_string.push_str(&format!("&page={page}"));
    }
    if let Some(limit) = limit {
        query_string.push_str(&format!("&limit={limit}"));
    }
    method = format!("{method}{query_string}");
    let timestamp = chrono::Utc::now().timestamp_millis();
    let res = client.get(&format!("{DOMAIN}{method}"))
        .header("X-COINEX-KEY", auth.get_key())
        .header("X-COINEX-SIGN", auth.sign(SignPayload::HTTP { verb: HTTPVerb::GET, method, body: None, timestamp }).0)
        .header("X-COINEX-TIMESTAMP", timestamp)
        .send().await;
    println!("{:#?}", res.unwrap().text().await);
}
