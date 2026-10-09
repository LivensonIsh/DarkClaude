#!/bin/bash
IDS=(
  "cognitivecomputations/dolphin-mistral-24b-venice-edition:free"
  "nousresearch/hermes-3-llama-3.1-8b"
  "nousresearch/hermes-3-llama-3.1-70b"
  "nousresearch/hermes-4-405b"
  "deepseek/deepseek-r1"
  "deepseek/deepseek-r1-distill-llama-8b"
  "qwen/qwen3.5-9b"
  "qwen/qwen3.8-max"
)
curl -s https://openrouter.ai/api/v1/models > /tmp/models.json
python3 - "${IDS[@]}" <<'PY'
import json,sys
d=json.load(open('/tmp/models.json'))['data']
idx={m['id']:m for m in d}
for i in sys.argv[1:]:
    m=idx.get(i)
    if not m: print(f"ABSENT   {i}"); continue
    p=m.get('pricing',{}); img='image' in str(m.get('architecture',{}).get('input_modalities',''))
    print(f"EXISTE   {i} | {p.get('prompt')}/{p.get('completion')} par token | ctx={m.get('context_length')} | image={img}")
PY
