pub fn create_sub_account() -> Result<http::Request<String>, Box<dyn std::error::Error>>{
    Ok(http::request::Builder::new()
        .method("GET")
        .uri("https://api.coinex.com/v2/account/subs")
        .body(String::new())?)
}
