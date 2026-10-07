use crate::{auth::{user_dto, AdminUser, AuthUser}, error::*, plans, AppState};
use axum::{extract::{Path, Query, State}, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub async fn list_plans(State(st): State<AppState>) -> Json<Value> {
    Json(json!({ "ok": true, "currency": "HTG", "fx": st.cfg.fx, "plans": st.plans.as_slice() }))
}

pub async fn pay_info(State(st): State<AppState>, _u: AuthUser) -> Json<Value> {
    Json(json!({ "ok": true, "manual": st.cfg.manual, "natcash": st.cfg.natcash, "moncash": st.cfg.moncash }))
}

#[derive(Deserialize)]
pub struct ManualReq { plan: Option<String>, method: Option<String>, reference: Option<String> }

pub async fn manual_request(State(st): State<AppState>, AuthUser(u, _): AuthUser, Json(b): Json<ManualReq>) -> ApiResult<Json<Value>> {
    if !st.cfg.manual { return Err(ApiError::new(503, "DISABLED", "Paiement manuel désactivé")); }
    let plan_id = b.plan.unwrap_or_default();
    let method = b.method.unwrap_or_default();
    let reference = b.reference.unwrap_or_default().trim().chars().take(80).collect::<String>();
    let plan = st.plans.iter().find(|p| p.id == plan_id && p.id != "free");
    let Some(plan) = plan else { return Err(ApiError::bad("Plan, méthode et référence de transaction requis")) };
    if (method != "natcash" && method != "moncash") || reference.len() < 4 { return Err(ApiError::bad("Plan, méthode et référence de transaction requis")); }
    let id: (i64,) = sqlx::query_as("INSERT INTO payments(user_id,plan,method,reference,amount_htg) VALUES($1,$2,$3,$4,$5) RETURNING id")
        .bind(u.id).bind(plan.id).bind(&method).bind(&reference).bind(plan.price_htg).fetch_one(&st.db).await?;
    Ok(Json(json!({ "ok": true, "paymentId": id.0, "message": "Paiement en attente de validation (quelques minutes à quelques heures)." })))
}

pub async fn cancel(State(st): State<AppState>, AuthUser(u, _): AuthUser) -> ApiResult<Json<Value>> {
    if u.plan != "starter" && u.plan != "unlimited" { return Err(ApiError::bad("Aucun abonnement mensuel à résilier.")); }
    let mut u = u;
    sqlx::query("UPDATE users SET cancel_at_period_end=true WHERE id=$1").bind(u.id).execute(&st.db).await?;
    u.cancel_at_period_end = true;
    Ok(Json(json!({ "ok": true, "user": user_dto(&u, &st) })))
}

#[derive(Deserialize)]
pub struct ContactBody { name: Option<String>, email: Option<String>, message: Option<String> }

pub async fn contact(State(st): State<AppState>, Json(b): Json<ContactBody>) -> ApiResult<Json<Value>> {
    let email = b.email.unwrap_or_default();
    let msg = b.message.unwrap_or_default();
    if !email.contains('@') || email.len() > 120 || msg.trim().len() < 5 { return Err(ApiError::bad("Email et message requis")); }
    sqlx::query("INSERT INTO contact_messages(name,email,message) VALUES($1,$2,$3)")
        .bind(b.name.unwrap_or_default().chars().take(80).collect::<String>()).bind(&email)
        .bind(msg.chars().take(4000).collect::<String>()).execute(&st.db).await?;
    Ok(Json(json!({ "ok": true })))
}

// ---------------- Admin ----------------
async fn activate<'e, E: sqlx::PgExecutor<'e>>(ex: E, uid: i64, plan: &str, days: i32) -> Result<(), sqlx::Error> {
    if plan == "lifetime" {
        sqlx::query("UPDATE users SET plan='lifetime', premium_until=NULL, cancel_at_period_end=false WHERE id=$1").bind(uid).execute(ex).await?;
    } else {
        sqlx::query("UPDATE users SET plan=$2, cancel_at_period_end=false, \
                     premium_until=GREATEST(COALESCE(CASE WHEN plan=$2 THEN premium_until END, now()), now()) + make_interval(days=>$3) WHERE id=$1")
            .bind(uid).bind(plan).bind(days).execute(ex).await?;
    }
    Ok(())
}

#[derive(sqlx::FromRow, Serialize)]
struct UserRow { id: i64, email: String, username: String, plan: String, premium_until: Option<DateTime<Utc>>, is_admin: bool, created_at: DateTime<Utc> }
#[derive(sqlx::FromRow, Serialize)]
struct PayRow { id: i64, user_id: i64, plan: String, method: String, reference: String, amount_htg: i64, status: String, created_at: DateTime<Utc>, email: String, username: String }

#[derive(Deserialize)]
pub struct SearchQ { search: Option<String> }
#[derive(Deserialize)]
pub struct StatusQ { status: Option<String> }
#[derive(Deserialize)]
pub struct SetPlan { plan: Option<String>, days: Option<i32> }

pub async fn admin_users(State(st): State<AppState>, _a: AdminUser, Query(q): Query<SearchQ>) -> ApiResult<Json<Value>> {
    let s = format!("%{}%", q.search.unwrap_or_default().to_lowercase());
    let rows: Vec<UserRow> = sqlx::query_as("SELECT id,email,username,plan,premium_until,is_admin,created_at FROM users WHERE lower(email) LIKE $1 OR lower(username) LIKE $1 ORDER BY id DESC LIMIT 100")
        .bind(s).fetch_all(&st.db).await?;
    Ok(Json(json!({ "ok": true, "users": rows })))
}

pub async fn admin_payments(State(st): State<AppState>, _a: AdminUser, Query(q): Query<StatusQ>) -> ApiResult<Json<Value>> {
    let rows: Vec<PayRow> = sqlx::query_as("SELECT p.id,p.user_id,p.plan,p.method,p.reference,p.amount_htg,p.status,p.created_at,u.email,u.username \
        FROM payments p JOIN users u ON u.id=p.user_id WHERE p.status=$1 ORDER BY p.id DESC LIMIT 200")
        .bind(q.status.unwrap_or_else(|| "pending".into())).fetch_all(&st.db).await?;
    Ok(Json(json!({ "ok": true, "payments": rows })))
}

pub async fn approve(State(st): State<AppState>, _a: AdminUser, Path(id): Path<i64>) -> ApiResult<Json<Value>> {
    let mut tx = st.db.begin().await?;
    let row: Option<(i64, String)> = sqlx::query_as("UPDATE payments SET status='approved', processed_at=now() WHERE id=$1 AND status='pending' RETURNING user_id, plan")
        .bind(id).fetch_optional(&mut *tx).await?;
    let Some((uid, plan)) = row else { return Err(ApiError::new(404, "NOT_FOUND", "Paiement introuvable ou déjà traité")) };
    activate(&mut *tx, uid, &plan, 30).await?;
    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn reject(State(st): State<AppState>, _a: AdminUser, Path(id): Path<i64>) -> ApiResult<Json<Value>> {
    sqlx::query("UPDATE payments SET status='rejected', processed_at=now() WHERE id=$1 AND status='pending'").bind(id).execute(&st.db).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn set_plan(State(st): State<AppState>, _a: AdminUser, Path(id): Path<i64>, Json(b): Json<SetPlan>) -> ApiResult<Json<Value>> {
    let plan = b.plan.unwrap_or_default();
    if !st.plans.iter().any(|p| p.id == plan) { return Err(ApiError::bad("Plan inconnu")); }
    if plan == "free" {
        sqlx::query("UPDATE users SET plan='free', premium_until=NULL WHERE id=$1").bind(id).execute(&st.db).await?;
    } else {
        activate(&st.db, id, &plan, b.days.unwrap_or(30).clamp(1, 3650)).await?;
    }
    Ok(Json(json!({ "ok": true })))
}
