use reqwest::{
    blocking::Response,
    header::{HeaderMap, HeaderValue},
};

pub fn get(uri: &str, token: Option<&str>) -> Response {
    let mut client = reqwest::blocking::Client::new()
        .get(uri)
        .headers(construct_headers());

    client = match token {
        Some(t) => client.header("authorization", format!("Bearer {}", t)),
        None => client,
    };

    let resp = client.send().unwrap();
    if !resp.status().is_success() {
        panic!(format!("Fetching {} failed: {:?}", uri, resp))
    }

    resp
}

pub fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.150 Safari/537.36"));
    headers
}
