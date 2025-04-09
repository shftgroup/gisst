use std::{collections::HashMap, str::FromStr};

use axum::http::header::HeaderMap;

use base64::{Engine, engine::general_purpose};

// Adapted from Rustus source: https://github.com/s3rius/rustus/blob/master/src/utils/headers.rs
pub fn check_header(header_map: &HeaderMap, header_name: &str, expr: fn(&str) -> bool) -> bool {
    header_map
        .get(header_name)
        .and_then(|header_val| header_val.to_str().ok().map(expr))
        .unwrap_or(false)
}

pub fn parse_header<T: FromStr>(header_map: &HeaderMap, header_name: &str) -> Option<T> {
    header_map
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .and_then(|val| val.parse::<T>().ok())
}

// Adapted from https://github.com/s3rius/rustus/blob/master/src/protocol/creation/routes.rs
pub fn get_metadata(headers: &HeaderMap) -> Option<HashMap<String, String>> {
    headers
        .get("Upload-Metadata")
        .and_then(|her| her.to_str().ok().map(String::from))
        .map(|header_string| {
            let mut meta_map = HashMap::new();
            for meta_pair in header_string.split(',') {
                let mut split = meta_pair.trim().split(' ');
                let key = split.next().map(String::from);
                let b64val = split.next().and_then(|b64val| {
                    general_purpose::STANDARD
                        .decode(b64val)
                        .ok()
                        .and_then(|value| String::from_utf8(value).ok())
                });
                if let (Some(key), Some(b64val)) = (key, b64val) {
                    meta_map.insert(key, b64val);
                }
            }
            meta_map
        })
}
