#!/bin/bash
: "${API:?}"; : "${SITE:?}"
echo "== DNS"; getent hosts "$API" || echo "DNS KO : ajoute l'enregistrement A chez dynv6"
echo "== Santé HTTPS"; curl -s -m 10 "https://$API/api/health" || echo "HTTPS KO (nginx/certbot)"
echo
grep -q CorsLayer ~/DarkClaude/server/src/main.rs || echo "ATTENTION : patch CORS absent du code, relance setup-api.sh"
cd ~/DarkClaude/server
ORIG="$SITE,https://darkclaude.online,https://www.darkclaude.online"
if grep -q "^CORS_ORIGINS" .env; then sed -i "s|^CORS_ORIGINS=.*|CORS_ORIGINS=$ORIG|" .env; else echo "CORS_ORIGINS=$ORIG" >> .env; fi
grep "^CORS_ORIGINS" .env
cd ~/DarkClaude/public
sed -i -E 's#var API = "https://[^"]*"#var API = "https://'"$API"'"#' js/api-base.js
grep "var API" js/api-base.js
cd ~/DarkClaude && bash seo.sh "$SITE"
tmux send-keys -t darkclaude C-c 2>/dev/null; sleep 2
tmux send-keys -t darkclaude "cd ~/DarkClaude/server && cargo run --release" Enter 2>/dev/null || echo "Relance le backend à la main"
echo "Attends 10 s que le backend démarre..."; sleep 10
echo "== Préflight"
curl -si -m 10 -X OPTIONS "https://$API/api/auth/register" -H "Origin: $SITE" -H "Access-Control-Request-Method: POST" -H "Access-Control-Request-Headers: content-type" | grep -i "HTTP/\|access-control"
