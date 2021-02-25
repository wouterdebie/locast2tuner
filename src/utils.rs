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

pub fn hdhr_checksum(device_id: usize) -> usize {
    let lookup_table: Vec<usize> = vec![
        0xA, 0x5, 0xF, 0x6, 0x7, 0xC, 0x1, 0xB, 0x9, 0x2, 0x8, 0xD, 0x4, 0x3, 0xE, 0x0,
    ];
    let mut checksum = 0;
    checksum ^= lookup_table[(device_id >> 28) & 0x0F];
    checksum ^= (device_id >> 24) & 0x0F;
    checksum ^= lookup_table[(device_id >> 20) & 0x0F];
    checksum ^= (device_id >> 16) & 0x0F;
    checksum ^= lookup_table[(device_id >> 12) & 0x0F];
    checksum ^= (device_id >> 8) & 0x0F;
    checksum ^= lookup_table[(device_id >> 4) & 0x0F];
    checksum ^= (device_id >> 0) & 0x0F;
    checksum
}
