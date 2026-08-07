#[derive(serde::Serialize, Debug)]
pub struct LoggedInUserInfo {
    pub email: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub initials: Option<String>,
    pub username: Option<String>,
    pub creator_id: uuid::Uuid,
    pub role: i32,
}
