// Most of this is based on https://github.com/maxcountryman/axum-login/blob/main/examples/oauth/src/main.rs
// We want to use Google Auth and hopefully OrcID OpenID


use std::env;
use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
    Extension
};

use uuid::Uuid;

use axum_login::{
    AuthUser,
    secrecy::SecretVec,
    PostgresStore
};
use axum_login::axum_sessions::extractors::{ReadableSession, WritableSession};
use sqlx::{PgConnection};

use oauth2::{
    CsrfToken,
    basic::BasicClient,
    AuthorizationCode,
    reqwest::async_http_client,
    Scope,
    AuthUrl,
    TokenUrl,
    ClientId,
    ClientSecret,
    RedirectUrl,
};
use serde::{Deserialize};
use crate::error::{AuthError, GISSTError};
use crate::server::ServerState;

// User attributes based on OpenID specification for "userinfo"
#[derive(Debug, Default, Clone, sqlx::FromRow)]
pub struct User {
    id: i32,
    creator_id: Option<Uuid>,
    password_hash: String,
    name: Option<String>, // OpenID
    given_name: Option<String>, //OpenID
    family_name: Option<String>, //OpenID
    preferred_username: Option<String>, //OpenID
    email: Option<String>, //OpenID
    picture: Option<String> //OpenID (this is a url string)
}

// Based on OpenID standard claims https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims
#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct OpenIDUserInfo {
    sub: String,
    name: Option<String>, // OpenID
    given_name: Option<String>, //OpenID
    family_name: Option<String>, //OpenID
    preferred_username: Option<String>, //OpenID
    email: Option<String>, //OpenID
    picture: Option<String> //OpenID (this is a url string)
}

impl AuthUser<i32> for User {
    fn get_id(&self) -> i32{
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.clone().into())
    }
}

impl User {
    async fn get_by_id(conn: &mut PgConnection, id:i32) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT
            id,
            creator_id,
            password_hash,
            name,
            given_name,
            family_name,
            preferred_username,
            email,
            picture
            FROM users WHERE id = $1"#,
            id
        )
            .fetch_optional(conn)
            .await
    }

    async fn insert(conn: &mut PgConnection, model: &User) -> Result<Self, GISSTError> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO users (creator_id, password_hash, name, given_name, family_name, preferred_username, email, picture)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
            id,
            creator_id,
            password_hash,
            name,
            given_name,
            family_name,
            preferred_username,
            email,
            picture
            "#,
            model.creator_id,
            model.password_hash,
            model.name,
            model.given_name,
            model.family_name,
            model.preferred_username,
            model.email,
            model.picture,
        )
            .fetch_one(conn)
            .await
            .map_err(|| AuthError::UserCreateError)
    }

    async fn update(conn: &mut PgConnection, model: &User) -> Result<Self, GISSTError> {
        sqlx::query_as!(
            Self,
            r#"
            UPDATE users SET
            creator_id = $1,
            password_hash = $2,
            name = $3,
            given_name = $4,
            family_name = $5,
            preferred_username = $6,
            email = $7,
            picture = $8
            WHERE id = $9
            RETURNING
            id,
            creator_id,
            password_hash,
            name,
            given_name,
            family_name,
            preferred_username,
            email,
            picture
            "#,
            model.creator_id,
            model.password_hash,
            model.name,
            model.given_name,
            model.family_name,
            model.preferred_username,
            model.email,
            model.picture,
            model.id
        )
            .fetch_one(conn)
            .await
            .map_err(|| AuthError::UserUpdateError)
    }

}

pub type AuthContext = axum_login::extractors::AuthContext<i32, User, PostgresStore<User>>;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
    state: CsrfToken
}

pub async fn oauth_callback_handler(
    mut auth: AuthContext,
    Query(query): Query<AuthRequest>,
    Extension(state): Extension<ServerState>,
    session: ReadableSession,
) -> impl IntoResponse {
    println!("Running oauth callback {query:?}");
    // Compare the csrf state in the callback with the state generated before the
    // request
    let original_csrf_state: CsrfToken = session.get("csrf_state").unwrap();
    let query_csrf_state = query.state.secret();
    let csrf_state_equal = original_csrf_state.secret() == query_csrf_state;

    drop(session);

    if !csrf_state_equal {
        println!("csrf state is invalid, cannot login",);

        // Return to some error
        return Redirect::to("/instances");
    }

    println!("Getting oauth token");
    // Get an auth token
    let token = state.oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .unwrap();

    // Get OpenID provider userinfo from token
    let profile = match reqwest::Client::new().get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(token.access_token().secret().to_owned())
        .send().await {
        Ok(res) => res,
        Err(e) => return Err(GISSTError::ReqwestError(e))
    };

    let profile: OpenIDUserInfo = profile.json::<OpenIDUserInfo>().await.unwrap();
    println!("Getting db connection");

    let mut conn = state.pool.acquire().await?;
    if let Some(user) = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", profile.email)
        .fetch_one(&conn)
        .await? {

        println!("Found user: {user:?} updating.");
        let user = User::update(&mut conn, &User{password_hash: token.access_token().secret().to_owned(), ..user}).await?;
        println!("Got user {user:?}. Logging in.");
        auth.login(&user).await.unwrap();
        println!("Logged in the user: {user:?}");
        Redirect::to("/instances")
    } else {
        println!("New user login, creating user record.");
        let user = User::insert(&mut conn, &User {
            id: 0, // will be ignored on insert since insert id is serial auto-increment
            creator_id: None,
            password_hash: token.access_token().secret().to_owned(), 
            name: profile.name,
            given_name: profile.given_name,
            family_name: profile.family_name,
            preferred_username: profile.preferred_username,
            email: profile.email,
            picture: profile.picture,
        }).await?;
        println!("User record created: {user:?}. Logging in.");
        auth.login(&user).await.unwrap();
        println!("Logged in the user: {user:?}");
        Redirect::to("/instances")
    }
}

pub async fn login_handler(
    Extension(state): Extension<ServerState>,
    mut session: WritableSession,
) -> impl IntoResponse {
    let (auth_url, csrf_state) = state.oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .url();

    session.insert("csrf_state", csrf_state).unwrap();

    Redirect::to(auth_url.as_ref())
}

pub async fn logout_handler(mut auth:AuthContext){
    dbg!("Logging out user: {}", &auth.current_user);
    auth.logout().await;
}

pub fn build_oauth_client() -> BasicClient {
    let client_id = env::var("GOOGLE_OAUTH_CLIENT_ID").expect("Missing GOOGLE_OAUTH_CLIENT_ID");
    let client_secret = env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("Missing GOOGLE_OAUTH_CLIENT_SECRET");
    let redirect_url = "http://localhost:3000/auth/google/callback".to_string();

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}