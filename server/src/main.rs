mod auth;
mod billing;
mod chat;
mod config;
mod error;
mod plans;

use axum::{extract::{DefaultBodyLimit, State}, http::{header, HeaderName, HeaderValue, Method, StatusCode}, routing::{get, post}, Json, Router};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub cfg: Arc<config::Config>,
    pub plans: Arc<Vec<plans::Plan>>,
    pub http: reqwest::Client,
}

async fn health(State(st): State<AppState>) -> (StatusCode, Json<Value>) {
    let db_ok = sqlx::query("SELECT 1").execute(&st.db).await.is_ok();
    let mut r = st.redis.clone();
    let rd_ok = redis::cmd("PING").query_async::<_, String>(&mut r).await.is_ok();
    if db_ok && rd_ok { (StatusCode::OK, Json(json!({ "ok": true }))) } else { (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "ok": false }))) }
}

async fn runtime(State(st): State<AppState>) -> Json<Value> {
    Json(json!({ "ok": true, "limits": { "maxMessageChars": st.cfg.max_chars } }))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt::init();
    let cfg = config::Config::from_env();
    if cfg.or_key.is_empty() { tracing::warn!("OPENROUTER_API_KEY est vide : le chat échouera."); }

    let db = PgPoolOptions::new().max_connections(10).connect(&cfg.database_url).await.expect("connexion PostgreSQL");
    sqlx::raw_sql(include_str!("../schema.sql")).execute(&db).await.expect("schéma SQL");
    let redis = redis::aio::ConnectionManager::new(redis::Client::open(cfg.redis_url.clone()).expect("REDIS_URL")).await.expect("connexion Redis");

    let state = AppState {
        db, redis,
        plans: Arc::new(plans::build_plans(&cfg)),
        http: reqwest::Client::builder().connect_timeout(Duration::from_secs(10)).build().expect("client http"),
        cfg: Arc::new(cfg.clone()),
    };

    let origins: Vec<HeaderValue> = std::env::var("CORS_ORIGINS").unwrap_or_default().split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, HeaderName::from_static("x-session-token")])
        .expose_headers([HeaderName::from_static("x-resolved-model"), HeaderName::from_static("x-chat-mode")])
        .max_age(Duration::from_secs(3600));

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/runtime", get(runtime))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/chat", post(chat::chat))
        .route("/api/billing/plans", get(billing::list_plans))
        .route("/api/billing/pay-info", get(billing::pay_info))
        .route("/api/billing/manual-request", post(billing::manual_request))
        .route("/api/billing/cancel-subscription", post(billing::cancel))
        .route("/api/contact", post(billing::contact))
        .route("/api/admin/users", get(billing::admin_users))
        .route("/api/admin/payments", get(billing::admin_payments))
        .route("/api/admin/payments/:id/approve", post(billing::approve))
        .route("/api/admin/payments/:id/reject", post(billing::reject))
        .route("/api/admin/users/:id/plan", post(billing::set_plan))
        .fallback_service(ServeDir::new(&cfg.public_dir))
        .layer(cors)
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
        .layer(SetResponseHeaderLayer::overriding(HeaderName::from_static("x-content-type-options"), HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::overriding(HeaderName::from_static("x-frame-options"), HeaderValue::from_static("DENY")))
        .layer(SetResponseHeaderLayer::overriding(HeaderName::from_static("referrer-policy"), HeaderValue::from_static("strict-origin-when-cross-origin")))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cfg.port)).await.expect("bind");
    tracing::info!("DarkClaude écoute sur le port {}", cfg.port);
    axum::serve(listener, app).await.expect("serveur");
}
