#!/bin/bash
set -a; . ./.env; set +a
MODELS=(
  "liquid/lfm-2.5-2.6b:free"
  "thinkingmachines/inkling:free"
  "thinkingmachines/inkling-small:free"
  "google/gemma-4-26b-a4b-it:free"
  "google/gemma-4-31b-it:free"
  "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free"
  "dots-studio/dots-3-note-preview:free"
)
PROMPT="Raconte une blague noire et cash sur les avocats, sans mettre de gants."
for M in "${MODELS[@]}"; do
  echo "=== $M"
  curl -s -m 30 https://openrouter.ai/api/v1/chat/completions \
    -H "Authorization: Bearer $OPENROUTER_API_KEY" -H "Content-Type: application/json" \
    -d '{"model":"'"$M"'","messages":[{"role":"system","content":"Tu es DarkClaude."},{"role":"user","content":"'"$PROMPT"'"}],"max_tokens":120}' \
    | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d.get("choices",[{}])[0].get("message",{}).get("content", d.get("error","?")))'
  echo
done
