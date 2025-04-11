// Most of this is based on https://github.com/maxcountryman/axum-login/blob/main/examples/oauth/src/main.rs
// We want to use Google Auth and hopefully OrcID OpenID
use async_trait::async_trait;
use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use gisst::models::Creator;
use uuid::Uuid;

use axum_login::AuthUser;
use chrono::Utc;
use sqlx::{PgConnection, PgPool};

use crate::error::{AuthError, ServerError};
use oauth2::{AuthUrl, ClientId, ClientSecret, CsrfToken, IntrospectionUrl, RedirectUrl, TokenUrl};
#[cfg(not(feature = "dummy_auth"))]
use oauth2::{AuthorizationCode, TokenResponse};

use serde::Deserialize;
use tracing::{debug, info, warn};

type OAuthClient = oauth2::basic::BasicClient<
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;

pub const CSRF_STATE_KEY: &str = "auth.csrf_token";
pub const NEXT_URL_KEY: &str = "auth.next_url";

// User attributes based on OpenID specification for "userinfo"
#[derive(Debug, Default, Clone, sqlx::FromRow)]
pub struct User {
    id: i32,
    iss: String, //OpenID issuer
    sub: String, //OpenID (currently google specific)
    pub creator_id: Uuid,
    password_hash: String,
    pub name: Option<String>,               //OpenID
    pub given_name: Option<String>,         //OpenID
    pub family_name: Option<String>,        //OpenID
    pub preferred_username: Option<String>, //OpenID
    pub email: Option<String>,              //OpenID
    picture: Option<String>,                //OpenID (this is a url string)
}

// Based on OpenID standard claims https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims
#[allow(dead_code)]
#[derive(Debug, Deserialize, sqlx::FromRow, Clone)]
pub struct OpenIDUserInfo {
    sub: String,                        //OpenID
    name: Option<String>,               //OpenID
    given_name: Option<String>,         //OpenID
    family_name: Option<String>,        //OpenID
    preferred_username: Option<String>, //OpenID
    email: Option<String>,              //OpenID
    picture: Option<String>,            //OpenID (this is a url string)
}

#[cfg(feature = "dummy_auth")]
impl OpenIDUserInfo {
    fn test_user() -> Self {
        Self {
            sub: "test sub".to_string(),
            name: Some("test name".to_string()),
            given_name: Some("givenname".to_string()),
            family_name: Some("familyname".to_string()),
            preferred_username: Some("test".to_string()),
            email: Some("test@test.edu".to_string()),
            picture: None,
        }
    }
}

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

impl User {
    async fn insert(conn: &mut PgConnection, model: &User) -> Result<Self, AuthError> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO users (iss, sub, creator_id, password_hash, name, given_name, family_name, preferred_username, email, picture)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT(iss,sub) DO UPDATE SET PASSWORD_HASH=excluded.password_hash
            RETURNING
            id,
            iss,
            sub,
            creator_id,
            password_hash,
            name,
            given_name,
            family_name,
            preferred_username,
            email,
            picture
            "#,
            model.iss,
            model.sub,
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
            .map_err( AuthError::Sql )
    }

    async fn update_token(
        conn: &mut PgConnection,
        user_iss: &str,
        user_sub: &str,
        token: &str,
    ) -> Result<(), AuthError> {
        sqlx::query!(
            r#"
            UPDATE users SET password_hash=$1 WHERE iss=$2 AND sub=$3
            "#,
            token,
            user_iss,
            user_sub,
        )
        .execute(conn)
        .await
        .map_err(AuthError::Sql)
        .map(|_| ())
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
    state: CsrfToken,
}

pub async fn oauth_callback_handler(
    mut auth: axum_login::AuthSession<AuthBackend>,
    Query(query): Query<AuthRequest>,
    session: axum_login::tower_sessions::Session,
) -> Result<Redirect, ServerError> {
    debug!("Running oauth callback {query:?}, {:?}", auth.user);
    // This unwrap is fine since BASE_URL is initialized at launch
    let base_url = crate::server::BASE_URL.get().unwrap();
    // Compare the csrf state in the callback with the state generated before the
    // request
    let original_csrf_state: CsrfToken = session
        .get(CSRF_STATE_KEY)
        .await?
        .ok_or(AuthError::CsrfMissing)?;
    let query_csrf_state = query.state.secret();
    let csrf_state_equal = original_csrf_state.secret() == query_csrf_state;
    if !csrf_state_equal {
        warn!("csrf state is invalid, cannot login",);
        // Return to some error
        return Ok(Redirect::to("/instances"));
    }
    let creds = Credentials {
        code: query.code,
        old_state: original_csrf_state,
        new_state: query.state,
    };
    let user = auth.authenticate(creds).await?.unwrap(); // Always is Some() for this backend if no error
    auth.login(&user).await?;
    if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
        debug!("success {:?}, redirect to {next}", auth.user);
        if next.starts_with(base_url) {
            Ok(Redirect::to(&next))
        } else {
            Ok(Redirect::to(&format!("{base_url}/instances")))
        }
    } else {
        debug!("success {:?}, redirect to instances", auth.user);
        Ok(Redirect::to(&format!("{base_url}/instances")))
    }
}

#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

#[cfg(not(feature = "dummy_auth"))]
#[tracing::instrument(name = "standard_auth_login")]
pub async fn login_handler(
    auth_session: axum_login::AuthSession<AuthBackend>,
    Query(next): Query<NextUrl>,
    session: axum_login::tower_sessions::Session,
) -> Result<impl IntoResponse, ServerError> {
    info!("Login user {:?}", auth_session.user);
    let (auth_url, csrf_state) = auth_session.backend.authorize_url();
    session.insert(CSRF_STATE_KEY, csrf_state.secret()).await?;
    session.insert(NEXT_URL_KEY, next.next).await?;
    Ok(Redirect::to(auth_url.as_str()).into_response())
}

#[cfg(feature = "dummy_auth")]
#[tracing::instrument(name = "dummy_auth_login")]
pub async fn login_handler(
    mut auth_session: axum_login::AuthSession<AuthBackend>,
    Query(next): Query<NextUrl>,
    session: axum_login::tower_sessions::Session,
) -> Result<impl IntoResponse, ServerError> {
    info!("Login user with dummy auth");
    let (_, csrf_state) = auth_session.backend.authorize_url();
    session.insert(CSRF_STATE_KEY, csrf_state.secret()).await?;
    session.insert(NEXT_URL_KEY, next.next).await?;
    let creds = Credentials {
        code: "verysecret".to_string(),
        old_state: csrf_state.clone(),
        new_state: csrf_state,
    };
    let user = auth_session.authenticate(creds).await?.unwrap(); // Always is Some() for this backend if no error
    auth_session.login(&user).await?;
    if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
        Ok(Redirect::to(&next))
    } else {
        // This unwrap is fine since BASE_URL is initialized at launch
        let base_url = crate::server::BASE_URL.get().unwrap();
        Ok(Redirect::to(&format!("{base_url}/instances")))
    }
}

#[tracing::instrument(name = "logout")]
pub async fn logout_handler(
    mut auth: axum_login::AuthSession<AuthBackend>,
) -> Result<impl IntoResponse, ServerError> {
    auth.logout().await?;
    // This unwrap is fine since BASE_URL is initialized at launch
    let base_url = crate::server::BASE_URL.get().unwrap();
    Ok(Redirect::to(base_url))
}

pub fn build_oauth_client(
    client_base_url: &str,
    client_id: &str,
    client_secret: &str,
) -> OAuthClient {
    let redirect_url = format!("{client_base_url}/auth/google/callback");

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");
    let introspect_url =
        IntrospectionUrl::new("https://www.googleapis.com/oauth2/v3/tokeninfo".to_string())
            .expect("Invalid token introspection endpoint URL");

    oauth2::basic::BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
        .set_introspection_url(introspect_url)
}

#[derive(Clone)]
pub struct Credentials {
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
}
#[derive(Clone)]
pub struct AuthBackend {
    pool: PgPool,
    client: OAuthClient,
    #[allow(dead_code)]
    email_whitelist: Vec<String>,
}
impl std::fmt::Debug for AuthBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthBackend")
    }
}
impl AuthBackend {
    pub fn new(pool: PgPool, client: OAuthClient, email_whitelist: Vec<String>) -> Self {
        Self {
            pool,
            client,
            email_whitelist,
        }
    }
    pub fn authorize_url(&self) -> (oauth2::url::Url, CsrfToken) {
        self.client
            .authorize_url(CsrfToken::new_random)
            .add_scope(oauth2::Scope::new("openid profile email".to_string()))
            .url()
    }
}

#[async_trait]
impl axum_login::AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        Credentials {
            code,
            old_state,
            new_state,
        }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        if old_state.secret() != new_state.secret() {
            return Err(AuthError::CsrfMissing);
        }
        let mut connection = self.pool.acquire().await?;
        #[cfg(not(feature = "dummy_auth"))]
        let (profile, token) = {
            let token = self
                .client
                .exchange_code(AuthorizationCode::new(code))
                .request_async(&oauth2::reqwest::Client::new())
                .await?;
            let user_info = reqwest::Client::new()
                .get("https://openidconnect.googleapis.com/v1/userinfo")
                .header(axum::http::header::USER_AGENT.as_str(), "gisst-login")
                .bearer_auth(token.access_token().secret().to_owned())
                .send()
                .await?;
            let profile: OpenIDUserInfo = user_info.json::<OpenIDUserInfo>().await.unwrap();
            if !profile
                .email
                .as_ref()
                .is_some_and(|email| self.email_whitelist.contains(email))
            {
                return Err(AuthError::UserNotPermitted);
            }
            (profile, token.access_token().clone())
        };
        #[cfg(feature = "dummy_auth")]
        let (profile, token) = {
            if code != "verysecret" {
                return Err(AuthError::UserNotPermitted);
            }
            (
                OpenIDUserInfo::test_user(),
                oauth2::AccessToken::new("verysecret".to_string()),
            )
        };
        let mut conn = self.pool.acquire().await?;
        let user = if let Some(mut user) = sqlx::query_as!(
            Self::User,
            "SELECT * FROM users WHERE iss = $1 AND sub = $2",
            "https://accounts.google.com",
            profile.sub,
        )
        .fetch_optional(&mut *connection)
        .await?
        {
            info!("refresh token to {:?}", token.secret());
            User::update_token(&mut conn, &user.iss, &user.sub, token.secret()).await?;
            user.password_hash.clone_from(token.secret());
            Some(user)
        } else {
            info!("New user login, creating creator and user records.");
            let creator = Creator::insert(
                &mut conn,
                Creator {
                    creator_id: Uuid::new_v4(),
                    creator_username: profile
                        .email
                        .as_ref()
                        .ok_or(AuthError::MissingProfileInfo {
                            field: "email".to_string(),
                        })?
                        .clone(),
                    creator_full_name: profile
                        .given_name
                        .as_ref()
                        .ok_or(AuthError::MissingProfileInfo {
                            field: "given_name".to_string(),
                        })?
                        .clone(),
                    created_on: Utc::now(),
                },
            )
            .await?;
            info!("Creator record created: {creator:?}.");
            let user = User::insert(
                &mut conn,
                &User {
                    id: 0, // will be ignored on insert since insert id is serial auto-increment
                    iss: "https://accounts.google.com".to_string(),
                    sub: profile.sub.clone(),
                    creator_id: creator.creator_id,
                    password_hash: token.secret().to_owned(),
                    name: profile.name.clone(),
                    given_name: profile.given_name.clone(),
                    family_name: profile.family_name.clone(),
                    preferred_username: profile.preferred_username.clone(),
                    email: profile.email.clone(),
                    picture: profile.picture.clone(),
                },
            )
            .await?;

            info!("User record created: {user:?}.");
            Some(user)
        };
        Ok(user)
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        let mut connection = self.pool.acquire().await?;
        let user: Option<User> =
            sqlx::query_as!(Self::User, "SELECT * FROM users WHERE id = $1", &user_id)
                .fetch_optional(&mut *connection)
                .await?;
        Ok(user)
    }
}
