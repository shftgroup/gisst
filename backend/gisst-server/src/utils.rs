use std::{collections::HashMap, str::FromStr};

use axum::headers::HeaderMap;

use base64::{engine::general_purpose, Engine};

// Adapted from Rustus source: https://github.com/s3rius/rustus/blob/master/src/utils/headers.rs
pub fn check_header(header_map: &HeaderMap, header_name: &str, expr: fn(&str) -> bool) -> bool {
    header_map
        .get(header_name)
        .and_then(|header_val| match header_val.to_str() {
            Ok(val) => Some(expr(val)),
            Err(_) => None,
        })
        .unwrap_or(false)
}

pub fn parse_header<T: FromStr>(header_map: &HeaderMap, header_name: &str) -> Option<T> {
    header_map
        .get(header_name)
        .and_then(|value| match value.to_str() {
            Ok(header_str) => Some(header_str),
            Err(_) => None,
        })
        .and_then(|val| match val.parse::<T>() {
            Ok(num) => Some(num),
            Err(_) => None,
        })
}

// Adapted from https://github.com/s3rius/rustus/blob/master/src/protocol/creation/routes.rs
pub fn get_metadata(headers: &HeaderMap) -> Option<HashMap<String, String>> {
    headers
        .get("Upload-Metadata")
        .and_then(|her| match her.to_str() {
            Ok(str_val) => Some(String::from(str_val)),
            Err(_) => None,
        })
        .map(|header_string| {
            let mut meta_map = HashMap::new();
            for meta_pair in header_string.split(',') {
                let mut split = meta_pair.trim().split(' ');
                let key = split.next();
                let b64val = split.next();
                if key.is_none() || b64val.is_none() {
                    continue;
                }
                let value = general_purpose::STANDARD
                    .decode(b64val.unwrap())
                    .map(|value| match String::from_utf8(value) {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    });
                if let Ok(Some(res)) = value {
                    meta_map.insert(String::from(key.unwrap()), res);
                }
            }
            meta_map
        })
}
