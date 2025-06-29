mod creator;
mod instance;
mod object;
pub mod players;
mod replay;
mod save;
mod screenshot;
mod state;
mod work;
mod dashboard;

pub use creator::router as creator_router;
pub use instance::router as instance_router;
pub use object::router as object_router;
pub use replay::router as replay_router;
pub use save::router as save_router;
pub use screenshot::router as screenshot_router;
pub use state::router as state_router;
pub use work::router as work_router;

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
