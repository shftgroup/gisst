mod creator;
mod instance;
mod object;
pub mod players;
mod replay;
mod save;
mod screenshot;
mod state;
mod work;

pub use creator::router as creator_router;
pub use instance::router as instance_router;
pub use object::router as object_router;
pub use replay::router as replay_router;
pub use save::router as save_router;
pub use screenshot::router as screenshot_router;
pub use state::router as state_router;
pub use work::router as work_router;

#[derive(Debug, serde::Deserialize)]
struct StateReplayPageQueryParams {
    state_page_num: Option<u32>,
    state_limit: Option<u32>,
    state_contains: Option<String>,
    replay_page_num: Option<u32>,
    replay_limit: Option<u32>,
    replay_contains: Option<String>,
    save_page_num: Option<u32>,
    save_limit: Option<u32>,
    save_contains: Option<String>,
    creator_id: Option<uuid::Uuid>,
}

#[derive(serde::Serialize, Debug)]
pub struct LoggedInUserInfo {
    email: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    username: Option<String>,
    creator_id: uuid::Uuid,
}

impl LoggedInUserInfo {
    pub fn generate_from_user(user: &crate::auth::User) -> LoggedInUserInfo {
        LoggedInUserInfo {
            email: user.email.clone(),
            name: user.name.clone(),
            given_name: user.given_name.clone(),
            family_name: user.family_name.clone(),
            username: user.preferred_username.clone(),
            creator_id: user.creator_id,
        }
    }
}
