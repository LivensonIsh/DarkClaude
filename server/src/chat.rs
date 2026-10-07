use crate::{auth::AuthUser, error::*, plans, AppState};
use axum::{body::Body, extract::State, http::{header, Response as HttpResponse}, response::Response, Json};
use bytes::Bytes;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{convert::Infallible, time::Duration};

const IDENTITY: &str = "Tu es DarkClaude, l'assistant de cette plateforme. Tu es une IA : ne prétends jamais être un humain.";
const OPENROUTER: &str = "https://openrouter.ai/api/v1/chat/completions";

// ---- Filtre minimal (plancher légal) ----
static MINOR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(enfants?|mineure?s?|minors?|child|children|kids?|underage|lolita|petite? fille|petit gar[cç]on|\d{1,2} ?(ans|years? old|yo))\b").unwrap());
static SEXUAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(sexe|sexuel|sex\b|porn|nude|\bnu[es]?\b|[ée]rot|baise|viol)").unwrap());
static CBRN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(synth[eè]se|synthesi[sz]e|fabriqu|manufactur|produc|make|build)[^.\n]{0,60}(sarin|\bVX\b|anthrax|nerve agent|agent neurotoxique|bioweapon|arme biologique|arme chimique|bombe nucl[eé]aire)").unwrap());
static RECENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(actualit|aujourd|hier|demain|derni|r[ée]cent|maintenant|en ce moment|cours du|prix|m[ée]t[ée]o|r[ée]sultat|news|today|latest|current|this week|20(2[5-9]|3\d))").unwrap());

fn blocked(t: &str) -> bool { (MINOR.is_match(t) && SEXUAL.is_match(t)) || CBRN.is_match(t) }

// ---- Retire les blocs <think>…</think> des modèles de raisonnement ----
#[derive(Default)]
struct ThinkFilter { in_think: bool, buf: String }
impl ThinkFilter {
    fn cut(&self, back: usize) -> usize {
        let mut k = self.buf.len().saturating_sub(back);
        while !self.buf.is_char_boundary(k) { k -= 1; }
        k
    }
    fn push(&mut self, s: &str) -> String {
        self.buf.push_str(s);
        let mut out = String::new();
        loop {
            if self.in_think {
                if let Some(i) = self.buf.find("</think>") {
                    self.buf = self.buf[i + 8..].to_string();
                    self.in_think = false;
                } else {
                    let k = self.cut(7);
                    self.buf = self.buf[k..].to_string();
                    break;
                }
            } else if let Some(i) = self.buf.find("<think>") {
                out.push_str(&self.buf[..i]);
                self.buf = self.buf[i + 7..].to_string();
                self.in_think = true;
            } else {
                let k = self.cut(6);
                out.push_str(&self.buf[..k]);
                self.buf = self.buf[k..].to_string();
                break;
            }
        }
        out
    }
    fn finish(&mut self) -> String { if self.in_think { String::new() } else { std::mem::take(&mut self.buf) } }
}

enum Ev { Chunk(Option<Result<Bytes, reqwest::Error>>), Tick }
fn nd(v: Value) -> Result<Bytes, Infallible> { Ok(Bytes::from(format!("{}\n", v))) }

#[derive(Deserialize)]
pub struct Hist { role: String, content: String }
#[derive(Deserialize)]
pub struct Img { #[serde(rename = "dataUrl")] data_url: String }
#[derive(Deserialize, Default)]
pub struct ChatBody {
    message: Option<String>, mode: Option<String>, hardness: Option<f64>, lang: Option<String>,
    stream: Option<bool>, history: Option<Vec<Hist>>, images: Option<Vec<Img>>,
}

fn valid_img(d: &str) -> bool {
    d.len() < 6_000_000 && ["data:image/png;base64,", "data:image/jpeg;base64,", "data:image/jpg;base64,", "data:image/webp;base64,", "data:image/gif;base64,"].iter().any(|p| d.starts_with(p))
}

async fn web_search(st: &AppState, q: &str) -> String {
    if st.cfg.search_key.is_empty() { return String::new(); }
    let r = st.http.post("https://api.tavily.com/search").bearer_auth(&st.cfg.search_key).timeout(Duration::from_secs(8))
        .json(&json!({ "query": q.chars().take(380).collect::<String>(), "max_results": 5, "search_depth": "basic" })).send().await;
    let Ok(r) = r else { return String::new() };
    let Ok(v) = r.json::<Value>().await else { return String::new() };
    v["results"].as_array().map(|a| a.iter().enumerate().map(|(i, x)| format!("[{}] {} ({})\n{}", i + 1,
        x["title"].as_str().unwrap_or(""), x["url"].as_str().unwrap_or(""), x["content"].as_str().unwrap_or("").chars().take(500).collect::<String>()))
        .collect::<Vec<_>>().join("\n\n")).unwrap_or_default()
}

async fn refund(mut r: redis::aio::ConnectionManager, key: String) {
    let _: redis::RedisResult<i64> = redis::cmd("DECR").arg(&key).query_async(&mut r).await;
}

pub async fn chat(State(st): State<AppState>, AuthUser(user, _): AuthUser, Json(b): Json<ChatBody>) -> ApiResult<Response> {
    let ChatBody { message, mode, hardness, lang, stream, history, images } = b;
    let message = message.unwrap_or_default().trim().to_string();
    let plan_id = plans::effective_plan(&user.plan, user.premium_until);
    let plan = plans::find(&st.plans, &plan_id).clone();

    let images: Vec<String> = images.unwrap_or_default().into_iter().take(1).map(|i| i.data_url).filter(|d| valid_img(d)).collect();
    if message.is_empty() && images.is_empty() { return Err(ApiError::new(400, "EMPTY", "Message vide")); }
    if message.chars().count() > st.cfg.max_chars { return Err(ApiError::new(400, "TOO_LONG", format!("Message trop long (max {})", st.cfg.max_chars))); }
    if blocked(&message) { return Err(ApiError::new(400, "BLOCKED", "Cette demande ne peut pas être traitée.")); }

    let mode = match mode.as_deref() { Some("code") => "code", Some("analyse") => "analyse", _ => "nolimit" };
    if mode == "analyse" && !plan.analyse { return Err(ApiError::new(403, "MODE_ANALYSE_LOCKED", "Mode Analyse réservé aux plans Premium.")); }
    if !images.is_empty() && !plan.vision { return Err(ApiError::new(402, "VISION_LOCKED", "L'analyse d'images est réservée aux plans Premium.")); }

    // Anti-rafale + quota journalier (jour d'Haïti ≈ UTC-5)
    let mut r = st.redis.clone();
    let rlk = format!("rl:{}", user.id);
    let rl: i64 = redis::cmd("INCR").arg(&rlk).query_async(&mut r).await?;
    if rl == 1 { let _: () = redis::cmd("EXPIRE").arg(&rlk).arg(60).query_async(&mut r).await?; }
    if rl > 20 { return Err(ApiError::new(429, "RATE_LIMIT", "Trop de messages, patiente une minute.")); }
    let day = (chrono::Utc::now() - chrono::Duration::hours(5)).format("%Y-%m-%d");
    let qk = format!("q:{}:{}", user.id, day);
    let used: i64 = redis::cmd("INCR").arg(&qk).query_async(&mut r).await?;
    if used == 1 { let _: () = redis::cmd("EXPIRE").arg(&qk).arg(172800).query_async(&mut r).await?; }
    if used > plan.daily {
        refund(r.clone(), qk.clone()).await;
        return Err(ApiError::new(402, "QUOTA_EXCEEDED", format!("Limite journalière atteinte ({} message{}). Passe à un plan supérieur.", plan.daily, if plan.daily > 1 { "s" } else { "" })));
    }

    let model: String = if !images.is_empty() {
        st.cfg.model_vision.clone()
    } else {
        match plan.tier { "premium" => st.cfg.model_premium.clone(), "standard" => st.cfg.model_standard.clone(), _ => st.cfg.model_free.clone() }
    };
    let lang_name = match lang.as_deref() { Some("en") => "English", Some("es") => "español", Some("it") => "italiano", Some("de") => "Deutsch", Some("ru") => "русский", _ => "français" };
    let mode_txt = match mode {
        "code" => "Tu es en mode Code : réponds avec du code propre, commenté et des explications courtes.",
        "analyse" => "Tu es en mode Analyse : raisonne par étapes, structure les faits, nuance et conclusions.",
        _ => "Réponds de façon directe et complète.",
    };
    let hard = hardness.unwrap_or(55.0).clamp(0.0, 100.0).round() as i64;
    let mut sys = format!("{IDENTITY}\n{mode_txt}\nRéponds en {lang_name}. Niveau de franchise : {hard}/100.");
    if plan.search && RECENT.is_match(&message) {
        let ctx = web_search(&st, &message).await;
        if !ctx.is_empty() { sys.push_str(&format!("\n\nRésultats web récents (cite les sources si utile) :\n{ctx}")); }
    }

    let mut messages: Vec<Value> = vec![json!({ "role": "system", "content": sys })];
    let hist = history.unwrap_or_default();
    let skip = hist.len().saturating_sub(24);
    for h in hist.into_iter().skip(skip) {
        if h.role == "user" || h.role == "assistant" {
            messages.push(json!({ "role": h.role, "content": h.content.chars().take(st.cfg.max_chars).collect::<String>() }));
        }
    }
    let user_content = if let Some(img) = images.first() {
        json!([{ "type": "text", "text": if message.is_empty() { "Analyse cette image." } else { message.as_str() } }, { "type": "image_url", "image_url": { "url": img } }])
    } else { json!(message) };
    messages.push(json!({ "role": "user", "content": user_content }));

    let want_stream = stream.unwrap_or(false);
    let mut rb = st.http.post(OPENROUTER).bearer_auth(&st.cfg.or_key).header("HTTP-Referer", st.cfg.base_url.as_str()).header("X-Title", "DarkClaude")
        .json(&json!({ "model": model, "messages": messages, "stream": want_stream, "temperature": 0.8 }));
    if !want_stream { rb = rb.timeout(Duration::from_secs(120)); }
    let resp = match rb.send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            refund(r_clone(&st), qk).await;
            return Err(ApiError::new(if r.status().as_u16() == 429 { 429 } else { 502 }, "UPSTREAM", "Le modèle est indisponible, réessaie."));
        }
        Err(_) => { refund(r_clone(&st), qk).await; return Err(ApiError::new(502, "UPSTREAM", "Le modèle est indisponible, réessaie.")); }
    };

    if !want_stream {
        let v: Value = resp.json().await.unwrap_or(Value::Null);
        let mut f = ThinkFilter::default();
        let mut out = f.push(v["choices"][0]["message"]["content"].as_str().unwrap_or(""));
        out.push_str(&f.finish());
        if out.trim().is_empty() { refund(r_clone(&st), qk).await; return Err(ApiError::new(502, "EMPTY_REPLY", "Réponse vide du modèle.")); }
        return Ok(HttpResponse::builder().status(200).header(header::CONTENT_TYPE, "application/json").header("x-resolved-model", "DarkClaude").header("x-chat-mode", mode)
            .body(Body::from(json!({ "ok": true, "reply": out }).to_string())).unwrap());
    }

    let rconn = st.redis.clone();
    let s = async_stream::stream! {
        let mut up = Box::pin(resp.bytes_stream());
        let mut buf: Vec<u8> = Vec::new();
        let mut filt = ThinkFilter::default();
        let (mut got, mut done, mut failed) = (false, false, false);
        let mut tick = tokio::time::interval_at(tokio::time::Instant::now() + Duration::from_secs(15), Duration::from_secs(15));
        loop {
            let ev = tokio::select! { c = up.next() => Ev::Chunk(c), _ = tick.tick() => Ev::Tick };
            match ev {
                Ev::Tick => { yield nd(json!({ "ping": 1 })); }
                Ev::Chunk(None) => break,
                Ev::Chunk(Some(Err(_))) => { failed = true; break; }
                Ev::Chunk(Some(Ok(bytes))) => {
                    buf.extend_from_slice(&bytes);
                    while let Some(pos) = buf.iter().position(|&x| x == b'\n') {
                        let raw: Vec<u8> = buf.drain(..=pos).collect();
                        let line = String::from_utf8_lossy(&raw).trim().to_string();
                        if let Some(data) = line.strip_prefix("data:") {
                            let data = data.trim();
                            if data == "[DONE]" { done = true; break; }
                            if let Ok(v) = serde_json::from_str::<Value>(data) {
                                if let Some(d) = v["choices"][0]["delta"]["content"].as_str() {
                                    let t = filt.push(d);
                                    if !t.is_empty() { got = true; yield nd(json!({ "delta": t })); }
                                }
                            }
                        }
                    }
                    if done { break; }
                }
            }
        }
        let rest = filt.finish();
        if !rest.is_empty() { got = true; yield nd(json!({ "delta": rest })); }
        if !got {
            refund(rconn.clone(), qk.clone()).await;
            if failed { yield nd(json!({ "error": "Le modèle est indisponible, réessaie.", "code": "UPSTREAM", "status": 502 })); }
            else { yield nd(json!({ "error": "Réponse vide du modèle.", "code": "EMPTY_REPLY", "status": 502 })); }
        }
    };
    Ok(HttpResponse::builder().status(200)
        .header(header::CONTENT_TYPE, "application/x-ndjson; charset=utf-8").header(header::CACHE_CONTROL, "no-store")
        .header("x-accel-buffering", "no").header("x-resolved-model", "DarkClaude").header("x-chat-mode", mode)
        .body(Body::from_stream(s)).unwrap())
}

fn r_clone(st: &AppState) -> redis::aio::ConnectionManager { st.redis.clone() }
