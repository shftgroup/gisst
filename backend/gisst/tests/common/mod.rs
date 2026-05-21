use uuid::Uuid;

// These UUIDs should match the values in ./fixtures/*.sql

pub fn env_id() -> Uuid{
    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
}
#[allow(dead_code)]
pub fn work_id() -> Uuid{
    Uuid::parse_str("00000000-0000-0000-0000-000000000010").unwrap()
}
#[allow(dead_code)]
pub fn file_id() -> Uuid{
    Uuid::parse_str("00000000-0000-0000-0000-000000000020").unwrap()
}
pub fn object_id() -> Uuid{
    Uuid::parse_str("00000000-0000-0000-0000-000000000030").unwrap()
}
pub fn instance_id() -> Uuid{
    Uuid::parse_str("00000000-0000-0000-0000-000000000040").unwrap()
}

