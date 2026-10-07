use crate::{error::*, plans, AppState};
use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use axum::{async_trait, extract::{FromRequestParts, State}, http::{header::HeaderMap, request::Parts}, Json};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(sqlx::FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub plan: String,
    pub premium_until: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub is_admin: bool,
}

pub fn sha256_hex(s: &str) -> String { hex::encode(Sha256::digest(s.as_bytes())) }

pub fn user_dto(u: &User, st: &AppState) -> Value {
    let plan = plans::effective_plan(&u.plan, u.premium_until);
    let p = plans::find(&st.plans, &plan);
    let premium_until = if plan == "lifetime" { Value::Null } else { json!(u.premium_until) };
    json!({
        "id": u.id.to_string(), "username": u.username, "name": u.username, "email": u.email,
        "plan": plan, "premium": plan != "free",
        "premiumUntil": premium_until,
        "admin": u.is_admin,
        "canCancelSubscription": (plan == "starter" || plan == "unlimited") && !u.cancel_at_period_end,
        "subscriptionCancelAtPeriodEnd": u.cancel_at_period_end,
        "dailyLimit": p.daily
    })
}

pub struct AuthUser(pub User, pub String);
pub struct AdminUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, st: &AppState) -> Result<Self, ApiError> {
        let tok = parts.headers.get("x-session-token").and_then(|v| v.to_str().ok()).unwrap_or("");
        if tok.is_empty() { return Err(ApiError::new(401, "NO_SESSION", "Non authentifié")); }
        let h = sha256_hex(tok);
        let u: Option<User> = sqlx::query_as(
            "SELECT u.id,u.email,u.username,u.password_hash,u.plan,u.premium_until,u.cancel_at_period_end,u.is_admin \
             FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.expires_at>now()")
            .bind(&h).fetch_optional(&st.db).await?;
        u.map(|u| AuthUser(u, h)).ok_or_else(|| ApiError::new(401, "BAD_SESSION", "Session expirée"))
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, st: &AppState) -> Result<Self, ApiError> {
        let AuthUser(u, _) = AuthUser::from_request_parts(parts, st).await?;
        if !u.is_admin { return Err(ApiError::new(403, "FORBIDDEN", "Admin requis")); }
        Ok(AdminUser(u))
    }
}

fn client_ip(h: &HeaderMap) -> String {
    h.get("x-forwarded-for").and_then(|v| v.to_str().ok()).and_then(|v| v.split(',').next()).map(|s| s.trim().to_string()).unwrap_or_else(|| "unknown".into())
}

async fn throttle(st: &AppState, headers: &HeaderMap) -> ApiResult<()> {
    let key = format!("ip:{}:auth", client_ip(headers));
    let mut r = st.redis.clone();
    let n: i64 = redis::cmd("INCR").arg(&key).query_async(&mut r).await?;
    if n == 1 { let _: () = redis::cmd("EXPIRE").arg(&key).arg(900).query_async(&mut r).await?; }
    if n > 20 { return Err(ApiError::new(429, "RATE_LIMIT", "Trop de tentatives, réessaie dans 15 minutes.")); }
    Ok(())
}

async fn new_session(st: &AppState, user_id: i64) -> ApiResult<String> {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    let token = hex::encode(b);
    sqlx::query("INSERT INTO sessions(token_hash,user_id,expires_at) VALUES($1,$2,now()+make_interval(days=>$3))")
        .bind(sha256_hex(&token)).bind(user_id).bind(st.cfg.session_days).execute(&st.db).await?;
    Ok(token)
}

#[derive(Deserialize)]
pub struct RegisterBody { email: Option<String>, username: Option<String>, password: Option<String> }
#[derive(Deserialize)]
pub struct LoginBody { login: Option<String>, password: Option<String> }

pub async fn register(State(st): State<AppState>, headers: HeaderMap, Json(b): Json<RegisterBody>) -> ApiResult<Json<Value>> {
    throttle(&st, &headers).await?;
    let email = b.email.unwrap_or_default().trim().to_lowercase();
    let username = b.username.unwrap_or_default().trim().to_string();
    let password = b.password.unwrap_or_default();
    let at = email.find('@');
    if email.len() > 120 || at.map_or(true, |i| i == 0 || !email[i..].contains('.') || email.contains(' ')) { return Err(ApiError::bad("Email invalide")); }
    if username.len() < 3 || username.len() > 24 || !username.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c)) {
        return Err(ApiError::bad("Nom d'utilisateur : 3 à 24 caractères (lettres, chiffres, _ . -)"));
    }
    if password.len() < 8 || password.len() > 200 { return Err(ApiError::bad("Mot de passe : 8 caractères minimum")); }
    let hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default().hash_password(password.as_bytes(), &salt).map(|h| h.to_string())
    }).await.map_err(|_| ApiError::internal())?.map_err(|_| ApiError::internal())?;
    let is_admin = st.cfg.admins.contains(&email);
    let res: Result<User, sqlx::Error> = sqlx::query_as(
        "INSERT INTO users(email,username,password_hash,is_admin) VALUES($1,$2,$3,$4) \
         RETURNING id,email,username,password_hash,plan,premium_until,cancel_at_period_end,is_admin")
        .bind(&email).bind(&username).bind(&hash).bind(is_admin).fetch_one(&st.db).await;
    match res {
        Ok(u) => { let token = new_session(&st, u.id).await?; Ok(Json(json!({ "ok": true, "token": token, "user": user_dto(&u, &st) }))) }
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(ApiError::new(409, "EXISTS", "Email ou nom d'utilisateur déjà utilisé")),
        Err(e) => Err(e.into()),
    }
}

pub async fn login(State(st): State<AppState>, headers: HeaderMap, Json(b): Json<LoginBody>) -> ApiResult<Json<Value>> {
    throttle(&st, &headers).await?;
    let login = b.login.unwrap_or_default().trim().to_lowercase();
    let password = b.password.unwrap_or_default();
    let u: Option<User> = sqlx::query_as(
        "SELECT id,email,username,password_hash,plan,premium_until,cancel_at_period_end,is_admin FROM users WHERE lower(email)=$1 OR lower(username)=$1")
        .bind(&login).fetch_optional(&st.db).await?;
    let mut u = u.ok_or_else(|| ApiError::new(401, "BAD_CREDENTIALS", "Identifiants incorrects"))?;
    let stored = u.password_hash.clone();
    let ok = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&stored).map(|p| Argon2::default().verify_password(password.as_bytes(), &p).is_ok()).unwrap_or(false)
    }).await.unwrap_or(false);
    if !ok { return Err(ApiError::new(401, "BAD_CREDENTIALS", "Identifiants incorrects")); }
    if st.cfg.admins.contains(&u.email) && !u.is_admin {
        sqlx::query("UPDATE users SET is_admin=true WHERE id=$1").bind(u.id).execute(&st.db).await?;
        u.is_admin = true;
    }
    let token = new_session(&st, u.id).await?;
    Ok(Json(json!({ "ok": true, "token": token, "user": user_dto(&u, &st) })))
}

pub async fn me(State(st): State<AppState>, AuthUser(u, _): AuthUser) -> Json<Value> {
    Json(json!({ "ok": true, "user": user_dto(&u, &st) }))
}

pub async fn logout(State(st): State<AppState>, AuthUser(_, h): AuthUser) -> ApiResult<Json<Value>> {
    sqlx::query("DELETE FROM sessions WHERE token_hash=$1").bind(h).execute(&st.db).await?;
    Ok(Json(json!({ "ok": true })))
}
