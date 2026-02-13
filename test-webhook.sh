#!/bin/bash
# Usage: ./test-webhook.sh <your-linear-webhook-secret>

ENDPOINT="https://linear-lark-bridge-production.up.railway.app/webhook"
SECRET="${1:?Usage: ./test-webhook.sh <your-LINEAR_WEBHOOK_SECRET>}"

echo "=== 1. Health check ==="
curl -s https://linear-lark-bridge-production.up.railway.app/health
echo -e "\n"

echo "=== 2. Missing signature → expect 401 ==="
curl -s -o /dev/null -w "HTTP Status: %{http_code}\n" \
  -X POST "$ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{}'
echo ""

echo "=== 3. Wrong signature → expect 401 ==="
curl -s -o /dev/null -w "HTTP Status: %{http_code}\n" \
  -X POST "$ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "linear-signature: deadbeef" \
  -d '{}'
echo ""

echo "=== 4. Ignored event type → expect 200, no Lark message ==="
PAYLOAD_IGNORE='{"action":"create","type":"Comment","url":"https://linear.app/test","data":{"id":"fake-001","title":"Ignored","priority":0,"identifier":"TEST-0","state":{"name":"Triage"},"assignee":null}}'
SIG_IGNORE=$(printf '%s' "$PAYLOAD_IGNORE" | openssl dgst -sha256 -hmac "$SECRET" | awk '{print $2}')
curl -s -o /dev/null -w "HTTP Status: %{http_code}\n" \
  -X POST "$ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "linear-signature: $SIG_IGNORE" \
  -d "$PAYLOAD_IGNORE"
echo ""

echo "=== 5. Valid Issue create (Urgent) → expect 200 + Lark card ==="
PAYLOAD_CREATE='{"action":"create","type":"Issue","url":"https://linear.app/team/issue/TEST-1","data":{"id":"fake-002","title":"Auth service returns 500 on login","priority":1,"identifier":"TEST-1","state":{"name":"In Progress"},"assignee":{"name":"QA Bot"}}}'
SIG_CREATE=$(printf '%s' "$PAYLOAD_CREATE" | openssl dgst -sha256 -hmac "$SECRET" | awk '{print $2}')
curl -s -o /dev/null -w "HTTP Status: %{http_code}\n" \
  -X POST "$ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "linear-signature: $SIG_CREATE" \
  -d "$PAYLOAD_CREATE"
echo ""

echo "=== 6. Valid Issue update (Medium, unassigned) → expect 200 + Lark card ==="
PAYLOAD_UPDATE='{"action":"update","type":"Issue","url":"https://linear.app/team/issue/TEST-2","data":{"id":"fake-003","title":"Update dashboard layout","priority":3,"identifier":"TEST-2","state":{"name":"Done"},"assignee":null}}'
SIG_UPDATE=$(printf '%s' "$PAYLOAD_UPDATE" | openssl dgst -sha256 -hmac "$SECRET" | awk '{print $2}')
curl -s -o /dev/null -w "HTTP Status: %{http_code}\n" \
  -X POST "$ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "linear-signature: $SIG_UPDATE" \
  -d "$PAYLOAD_UPDATE"
echo ""

echo "=== Done. Check Railway logs and your Lark group. ==="
