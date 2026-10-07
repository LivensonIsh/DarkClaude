use crate::config::Config;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Plan {
    pub id: &'static str,
    pub label: &'static str,
    #[serde(rename = "priceHtg")]
    pub price_htg: i64,
    pub period: Option<&'static str>,
    #[serde(rename = "dailyMessages")]
    pub daily: i64,
    #[serde(skip)]
    pub tier: &'static str,
    pub vision: bool,
    pub search: bool,
    pub analyse: bool,
}

pub fn build_plans(cfg: &Config) -> Vec<Plan> {
    let htg = |eur: f64| ((eur * cfg.fx / 50.0).round() * 50.0) as i64;
    vec![
        Plan { id: "free", label: "Gratuit", price_htg: 0, period: None, daily: 2, tier: "free", vision: false, search: false, analyse: false },
        Plan { id: "starter", label: "Standard", price_htg: htg(9.9), period: Some("month"), daily: 60, tier: "standard", vision: false, search: false, analyse: false },
        Plan { id: "unlimited", label: "Premium", price_htg: htg(20.0), period: Some("month"), daily: 300, tier: "premium", vision: true, search: true, analyse: true },
        Plan { id: "lifetime", label: "Lifetime", price_htg: htg(cfg.lifetime_eur), period: Some("lifetime"), daily: 1000, tier: "premium", vision: true, search: true, analyse: true },
    ]
}

pub fn find<'a>(plans: &'a [Plan], id: &str) -> &'a Plan {
    plans.iter().find(|p| p.id == id).unwrap_or(&plans[0])
}

/// Plan réellement actif (un abonnement mensuel expiré retombe en « free »).
pub fn effective_plan(plan: &str, premium_until: Option<DateTime<Utc>>) -> String {
    if plan != "free" && plan != "lifetime" {
        if let Some(t) = premium_until {
            if t < Utc::now() { return "free".into(); }
        }
    }
    match plan { "starter" | "unlimited" | "lifetime" => plan.to_string(), _ => "free".into() }
}
