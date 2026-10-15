#!/usr/bin/env bash
# Seed demo data through the live API: a doctor, a patient, an OPD appointment,
# an issued invoice with line items, and a payment.
#
# Requires: the server running, AWS CLI logged in to the pool's profile, and the
# values in .env. Idempotent-ish: re-running just creates more rows.
#
# Usage:  ./scripts/seed.sh
# Env overrides: API_BASE, SEED_ADMIN_EMAIL, SEED_ADMIN_PASS
set -euo pipefail
cd "$(dirname "$0")/.."

API="${API_BASE:-http://localhost:8080}/api/v1"
POOL="$(grep -E '^COGNITO_USER_POOL_ID=' .env | cut -d= -f2-)"
PROFILE="$(grep -E '^AWS_PROFILE=' .env | cut -d= -f2- || true)"; PROFILE="${PROFILE:-project-d}"
ADMIN_EMAIL="${SEED_ADMIN_EMAIL:-seed-admin@projectd.dev}"
ADMIN_PASS="${SEED_ADMIN_PASS:-S3cret!pass}"

get() { python3 -c "import sys,json;print(json.load(sys.stdin).get('$1',''))"; }

echo "API=$API  pool=$POOL  profile=$PROFILE"

echo "==> ensure admin user ($ADMIN_EMAIL) exists, confirmed, in super_admin"
curl -s -X POST "$API/auth/register" -H 'content-type: application/json' \
  -d "{\"email\":\"$ADMIN_EMAIL\",\"password\":\"$ADMIN_PASS\",\"given_name\":\"Seed\",\"family_name\":\"Admin\"}" >/dev/null || true
aws cognito-idp admin-confirm-sign-up  --user-pool-id "$POOL" --username "$ADMIN_EMAIL" --profile "$PROFILE" >/dev/null 2>&1 || true
aws cognito-idp admin-add-user-to-group --user-pool-id "$POOL" --username "$ADMIN_EMAIL" --group-name super_admin --profile "$PROFILE" >/dev/null 2>&1 || true

echo "==> login"
TOKEN="$(curl -s -X POST "$API/auth/login" -H 'content-type: application/json' \
  -d "{\"email\":\"$ADMIN_EMAIL\",\"password\":\"$ADMIN_PASS\"}" | get access_token)"
[ -n "$TOKEN" ] || { echo "ERROR: login failed"; exit 1; }
AUTH=(-H "authorization: Bearer $TOKEN")

echo "==> create doctor"
DOC_ID="$(curl -s -X POST "$API/doctors" "${AUTH[@]}" -H 'content-type: application/json' \
  -d '{"first_name":"Meena","last_name":"Iyer","specialization":"General Medicine","department":"OPD","consultation_fee":"500.00"}' | get id)"
echo "   doctor=$DOC_ID"

echo "==> create patient"
PAT_ID="$(curl -s -X POST "$API/patients" "${AUTH[@]}" -H 'content-type: application/json' \
  -d '{"first_name":"Asha","last_name":"Rao","gender":"female","phone":"+919000000001","email":"asha@example.com"}' | get id)"
echo "   patient=$PAT_ID"

echo "==> book appointment (tomorrow 10:00 UTC)"
WHEN="$(python3 -c "import datetime;print((datetime.datetime.utcnow()+datetime.timedelta(days=1)).strftime('%Y-%m-%dT10:00:00Z'))")"
APP_ID="$(curl -s -X POST "$API/appointments" "${AUTH[@]}" -H 'content-type: application/json' \
  -d "{\"patient_id\":\"$PAT_ID\",\"doctor_id\":\"$DOC_ID\",\"scheduled_at\":\"$WHEN\",\"reason\":\"Fever and cough\"}" | get id)"
echo "   appointment=$APP_ID"

echo "==> create issued invoice (consultation + lab, +tax)"
INV_ID="$(curl -s -X POST "$API/invoices" "${AUTH[@]}" -H 'content-type: application/json' \
  -d "{\"patient_id\":\"$PAT_ID\",\"appointment_id\":\"$APP_ID\",\"issue_now\":true,\"tax\":\"40.00\",\"items\":[{\"description\":\"Consultation\",\"quantity\":\"1\",\"unit_price\":\"500.00\"},{\"description\":\"CBC Test\",\"quantity\":\"1\",\"unit_price\":\"300.00\"}]}" | get id)"
echo "   invoice=$INV_ID"

echo "==> record full payment (UPI 840.00)"
curl -s -X POST "$API/invoices/$INV_ID/payments" "${AUTH[@]}" -H 'content-type: application/json' \
  -d '{"amount":"840.00","method":"upi","reference":"UPI-123"}' | python3 -c "import sys,json;d=json.load(sys.stdin);print('   invoice status:', d['invoice']['status'], '| paid:', d['invoice']['amount_paid'])"

echo "==> done.  doctor=$DOC_ID patient=$PAT_ID appointment=$APP_ID invoice=$INV_ID"
