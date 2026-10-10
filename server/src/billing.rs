use crate::{auth::{user_dto, AdminUser, AuthUser}, error::*, AppState};
use chrono::{DateTime, Utc};
use axum::{extract::{Path, Query, State}, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub async fn list_plans(State(st): State<AppState>) -> Json<Value> {
    Json(json!({ "ok": true, "currency": "HTG", "fx": st.cfg.fx, "plans": st.plans.as_slice() }))
}

pub async fn pay_info(State(st): State<AppState>, _u: AuthUser) -> Json<Value> {
    Json(json!({ "ok": true, "manual": st.cfg.manual, "natcash": st.cfg.natcash, "moncash": st.cfg.moncash }))
}

#[derive(Deserialize)]
pub struct ManualReq {
    plan: Option<String>, method: Option<String>, reference: Option<String>,
    proof_data_url: Option<String>, sender_number: Option<String>, sender_is_agent: Option<bool>, notes: Option<String>,
}

pub async fn manual_request(State(st): State<AppState>, AuthUser(u, _): AuthUser, Json(b): Json<ManualReq>) -> ApiResult<Json<Value>> {
    if !st.cfg.manual { return Err(ApiError::new(503, "DISABLED", "Paiement manuel désactivé")); }
    let plan_id = b.plan.unwrap_or_default();
    let method = b.method.unwrap_or_default();
    let reference = b.reference.unwrap_or_default().trim().chars().take(80).collect::<String>();
    let proof = b.proof_data_url.unwrap_or_default();
    let sender_number = b.sender_number.unwrap_or_default().trim().chars().take(30).collect::<String>();
    let notes = b.notes.unwrap_or_default().trim().chars().take(500).collect::<String>();
    let plan = st.plans.iter().find(|p| p.id == plan_id && p.id != "free");
    let Some(plan) = plan else { return Err(ApiError::bad("Plan, méthode et référence de transaction requis")) };
    if (method != "natcash" && method != "moncash") || reference.len() < 4 { return Err(ApiError::bad("Plan, méthode et référence de transaction requis")); }
    if proof.len() < 20 || proof.len() > 6_000_000 || !proof.starts_with("data:image/") { return Err(ApiError::bad("La capture d'écran de preuve de paiement est obligatoire")); }
    if sender_number.chars().count() < 8 { return Err(ApiError::bad("Le numéro qui a transféré l'argent est obligatoire")); }
    let Some(sender_is_agent) = b.sender_is_agent else { return Err(ApiError::bad("Indique si c'est toi ou un agent qui a transféré l'argent")) };
    let id: (i64,) = sqlx::query_as("INSERT INTO payments(user_id,plan,method,reference,amount_htg,proof_data_url,sender_number,sender_is_agent,notes) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING id")
        .bind(u.id).bind(plan.id).bind(&method).bind(&reference).bind(plan.price_htg)
        .bind(&proof).bind(&sender_number).bind(sender_is_agent).bind(&notes)
        .fetch_one(&st.db).await?;
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
struct PayRow { id: i64, user_id: i64, plan: String, method: String, reference: String, amount_htg: i64, status: String, created_at: DateTime<Utc>, email: String, username: String, proof_data_url: Option<String>, sender_number: Option<String>, sender_is_agent: bool, notes: Option<String> }

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
    let rows: Vec<PayRow> = sqlx::query_as("SELECT p.id,p.user_id,p.plan,p.method,p.reference,p.amount_htg,p.status,p.created_at,u.email,u.username,p.proof_data_url,p.sender_number,p.sender_is_agent,p.notes \
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

#[derive(sqlx::FromRow, Serialize)]
pub struct MsgRow { id: i64, role: String, content: String, created_at: DateTime<Utc> }

#[derive(Deserialize)]
pub struct HistQuery { conversation_id: Option<i64> }

pub async fn history(State(st): State<AppState>, AuthUser(u, _): AuthUser, Query(q): Query<HistQuery>) -> ApiResult<Json<Value>> {
    let rows: Vec<MsgRow> = if let Some(cid) = q.conversation_id {
        sqlx::query_as("SELECT id,role,content,created_at FROM messages WHERE user_id=$1 AND conversation_id=$2 ORDER BY created_at ASC LIMIT 300")
            .bind(u.id).bind(cid).fetch_all(&st.db).await?
    } else {
        sqlx::query_as("SELECT id,role,content,created_at FROM messages WHERE user_id=$1 ORDER BY created_at ASC LIMIT 300")
            .bind(u.id).fetch_all(&st.db).await?
    };
    Ok(Json(json!({ "ok": true, "messages": rows })))
}

#[derive(sqlx::FromRow, Serialize)]
pub struct ConvRow { id: i64, title: String, created_at: DateTime<Utc> }

pub async fn list_conversations(State(st): State<AppState>, AuthUser(u, _): AuthUser) -> ApiResult<Json<Value>> {
    let rows: Vec<ConvRow> = sqlx::query_as("SELECT id,title,created_at FROM conversations WHERE user_id=$1 ORDER BY created_at DESC LIMIT 50")
        .bind(u.id).fetch_all(&st.db).await?;
    Ok(Json(json!({ "ok": true, "conversations": rows })))
}

#[derive(Deserialize)]
pub struct NewConvBody { title: Option<String> }

pub async fn create_conversation(State(st): State<AppState>, AuthUser(u, _): AuthUser, Json(b): Json<NewConvBody>) -> ApiResult<Json<Value>> {
    let title = b.title.unwrap_or_else(|| "Conversation".into());
    let row: (i64,) = sqlx::query_as("INSERT INTO conversations(user_id, title) VALUES ($1,$2) RETURNING id")
        .bind(u.id).bind(title.chars().take(80).collect::<String>()).fetch_one(&st.db).await?;
    Ok(Json(json!({ "ok": true, "id": row.0 })))
}

pub async fn delete_conversation(State(st): State<AppState>, AuthUser(u, _): AuthUser, Path(id): Path<i64>) -> ApiResult<Json<Value>> {
    sqlx::query("DELETE FROM conversations WHERE id=$1 AND user_id=$2").bind(id).bind(u.id).execute(&st.db).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn clear_history(State(st): State<AppState>, AuthUser(u, _): AuthUser) -> ApiResult<Json<Value>> {
    sqlx::query("DELETE FROM messages WHERE user_id=$1").bind(u.id).execute(&st.db).await?;
    Ok(Json(json!({ "ok": true })))
}
