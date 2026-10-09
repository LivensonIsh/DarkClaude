#!/bin/bash
set -a; . ./.env; set +a
if [ -n "$OPENROUTER_API_KEY" ]; then echo "Clé : présente (${#OPENROUTER_API_KEY} caractères)"; else echo "Clé : VIDE"; fi
for M in "$MODEL_FREE" "$MODEL_STANDARD" "$MODEL_PREMIUM"; do
  echo "=== $M"
  curl -s -m 40 -o /tmp/or.json -w "HTTP %{http_code}\n" https://openrouter.ai/api/v1/chat/completions \
    -H "Authorization: Bearer $OPENROUTER_API_KEY" -H "Content-Type: application/json" \
    -d '{"model":"'"$M"'","messages":[{"role":"user","content":"Dis bonjour"}],"max_tokens":20}'
  head -c 300 /tmp/or.json; echo
done
