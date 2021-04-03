use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use regex::Regex;
use reqwest::{
    blocking::Response,
    header::{HeaderMap, HeaderValue},
    Url,
};

pub trait Or {
    /// Return `self` if it's not empty, otherwise `other`
    fn or<'a>(&'a self, other: &'a str) -> &str;
}

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

pub async fn get_async(uri: &str, token: Option<&str>) -> reqwest::Response {
    let mut client = reqwest::Client::new().get(uri).headers(construct_headers());
    client = match token {
        Some(t) => client.header("authorization", format!("Bearer {}", t)),
        None => client,
    };

    let resp = client.send().await.unwrap();
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

pub fn name_only(value: &str) -> &str {
    match Regex::new(r"\d+\.\d+ (.+)").unwrap().captures(value) {
        Some(c) => c.get(1).map_or("", |m| m.as_str()),
        None => &value,
    }
}

pub fn format_time(timestamp: i64) -> String {
    let naive = NaiveDateTime::from_timestamp(timestamp / 1000, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%Y%m%d%H%M%S %z").to_string()
}

pub fn format_date(timestamp: i64) -> String {
    let naive = NaiveDateTime::from_timestamp(timestamp / 1000, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%Y%m%d").to_string()
}

pub fn split(value: &str, sep: &str) -> Vec<String> {
    value.split(sep).map(|x| x.to_string()).collect()
}

pub fn format_time_local_iso(timestamp: i64, timezone: &Tz) -> String {
    let naive = NaiveDateTime::from_timestamp(timestamp / 1000, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    let in_timezone = datetime.with_timezone(timezone);
    in_timezone.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_date_iso(timestamp: i64) -> String {
    let naive = NaiveDateTime::from_timestamp(timestamp / 1000, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%F").to_string()
}

const HD: [&'static str; 3] = ["1080", "720", "HDTV"];
pub fn aspect_ratio(properties: &str) -> String {
    for hd in HD.iter() {
        if properties.contains(hd) {
            return "16:9".to_string();
        }
    }
    "4:3".to_string()
}

pub fn quality(properties: &str) -> String {
    if properties.contains("HDTV") {
        return "HDTV".to_string();
    } else {
        return "SD".to_string();
    }
}

pub fn base_url(mut url: Url) -> Url {
    url.path_segments_mut().unwrap().clear();
    url.set_query(None);
    url
}
