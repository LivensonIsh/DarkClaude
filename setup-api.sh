#!/bin/bash
set -e
: "${API:?API manquant}"; : "${SITE:?SITE manquant}"
getent hosts "$API" || { echo "DNS pas encore prêt pour $API"; exit 1; }
sudo apt install -y nginx certbot python3-certbot-nginx < /dev/null
sudo ufw allow 80,443/tcp < /dev/null || true
cat <<'NGINX' | sudo tee /etc/nginx/sites-available/darkclaude-api > /dev/null
server {
  server_name API_HOST;
  location / {
    proxy_pass http://127.0.0.1:8585;
    proxy_http_version 1.1;
    proxy_buffering off;
    proxy_read_timeout 300s;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
  }
}
NGINX
sudo sed -i "s/API_HOST/$API/" /etc/nginx/sites-available/darkclaude-api
sudo ln -sf /etc/nginx/sites-available/darkclaude-api /etc/nginx/sites-enabled/
sudo nginx -t < /dev/null
sudo systemctl reload nginx
sudo certbot --nginx -d "$API" --non-interactive --agree-tos -m milienlivenson44@gmail.com < /dev/null
cd ~/DarkClaude/server
sed -i 's/features = \["fs", "set-header", "trace"\]/features = ["fs", "set-header", "trace", "cors"]/' Cargo.toml
if ! grep -q CorsLayer src/main.rs; then
python3 - <<'PY'
p='src/main.rs'; s=open(p,encoding='utf-8').read()
s=s.replace('http::{HeaderName, HeaderValue, StatusCode}','http::{header, HeaderName, HeaderValue, Method, StatusCode}')
s=s.replace('    let app = Router::new()','''    let origins: Vec<HeaderValue> = std::env::var("CORS_ORIGINS").unwrap_or_default().split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, HeaderName::from_static("x-session-token")])
        .expose_headers([HeaderName::from_static("x-resolved-model"), HeaderName::from_static("x-chat-mode")])
        .max_age(Duration::from_secs(3600));

    let app = Router::new()''',1)
s=s.replace('        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))','        .layer(cors)\n        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))',1)
open(p,'w',encoding='utf-8').write(s)
PY
fi
ORIG="$SITE,https://darkclaude.online,https://www.darkclaude.online"
if grep -q "^CORS_ORIGINS" .env; then sed -i "s|^CORS_ORIGINS=.*|CORS_ORIGINS=$ORIG|" .env; else echo "CORS_ORIGINS=$ORIG" >> .env; fi
cargo build --release 2>&1 | grep -E "^error|Finished"
sudo ufw delete allow 8585/tcp < /dev/null || true
echo "TERMINÉ"
