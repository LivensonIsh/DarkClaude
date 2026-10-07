use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub fx: f64,
    pub lifetime_eur: f64,
    pub max_chars: usize,
    pub or_key: String,
    pub search_key: String,
    pub model_free: String,
    pub model_standard: String,
    pub model_premium: String,
    pub model_vision: String,
    pub manual: bool,
    pub natcash: String,
    pub moncash: String,
    pub admins: Vec<String>,
    pub session_days: i32,
    pub public_dir: String,
    pub base_url: String,
    pub database_url: String,
    pub redis_url: String,
}

fn get(k: &str, d: &str) -> String {
    env::var(k).ok().filter(|v| !v.is_empty()).unwrap_or_else(|| d.to_string())
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            port: get("PORT", "8585").parse().unwrap_or(8585),
            fx: get("FX_EUR_HTG", "150").parse().unwrap_or(150.0),
            lifetime_eur: get("LIFETIME_EUR", "69").parse().unwrap_or(69.0),
            max_chars: get("MAX_MESSAGE_CHARS", "4000").parse().unwrap_or(4000),
            or_key: get("OPENROUTER_API_KEY", ""),
            search_key: get("SEARCH_API_KEY", ""),
            model_free: get("MODEL_FREE", "openrouter/free"),
            model_standard: get("MODEL_STANDARD", "cognitivecomputations/dolphin-mistral-24b-venice-edition"),
            model_premium: get("MODEL_PREMIUM", "cognitivecomputations/dolphin3.0-r1-mistral-24b"),
            model_vision: get("MODEL_VISION", "qwen/qwen3.8-27b:free"),
            manual: get("ENABLE_MANUAL_ACTIVATION", "true") != "false",
            natcash: get("PAY_NATCASH_NUMBER", ""),
            moncash: get("PAY_MONCASH_NUMBER", ""),
            admins: get("ADMIN_EMAILS", "")
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
            session_days: get("SESSION_TTL_DAYS", "30").parse().unwrap_or(30),
            public_dir: get("PUBLIC_DIR", "../public"),
            base_url: get("APP_BASE_URL", "https://darkclaude.online"),
            database_url: get("DATABASE_URL", ""),
            redis_url: get("REDIS_URL", "redis://127.0.0.1:6379"),
        }
    }
}
