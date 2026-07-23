#!/usr/bin/env bash

set -euo pipefail

LIB_FILE="peerx-contracts/counter/src/lib.rs"
BATCH_FILE="peerx-contracts/counter/batch.rs"

SENSITIVE_ENTRYPOINTS=(
  swap
  safe_swap
  add_liquidity
  remove_liquidity
  stake
  claim_staking_bonuses
  claim_stake
  unstake_early
  pool_add_liquidity
  pool_remove_liquidity
  pool_swap
)

extract_function_block() {
  local file="$1"
  local function_name="$2"

  awk -v fn="$function_name" '
    $0 ~ "^[[:space:]]*pub fn " fn "\\(" { capture = 1 }
    capture && $0 ~ "^[[:space:]]*pub fn " && $0 !~ "^[[:space:]]*pub fn " fn "\\(" { exit }
    capture { print }
  ' "$file"
}

assert_contains_guard() {
  local function_name="$1"
  local function_body

  function_body="$(extract_function_block "$LIB_FILE" "$function_name")"

  if [[ -z "$function_body" ]]; then
    echo "Missing function block for ${function_name} in ${LIB_FILE}" >&2
    exit 1
  fi

  if ! grep -Eq 'require_authenticated_verified_user|require_verified_user' <<<"$function_body"; then
    echo "Sensitive entry point ${function_name} is missing a shared KYC guard" >&2
    exit 1
  fi
}

for function_name in "${SENSITIVE_ENTRYPOINTS[@]}"; do
  assert_contains_guard "$function_name"
done

if ! grep -q 'crate::require_verified_user(env, &user)' "$BATCH_FILE"; then
  echo "Batch KYC guard is missing the shared verified-user check" >&2
  exit 1
fi

if ! grep -q 'user.require_auth();' "$BATCH_FILE"; then
  echo "Batch KYC guard is missing user authentication enforcement" >&2
  exit 1
fi

if [[ "$(grep -c 'authorize_batch_access(env, &operations)' "$BATCH_FILE")" -lt 2 ]]; then
  echo "Batch execution paths are missing the shared authorization helper" >&2
  exit 1
fi

# `MintToken` is a privileged, admin-only batch operation (issue #39). The
# shared authorization helper must gate any batch that mints behind the
# contract admin, so a MintToken op can no longer bypass KYC/admin checks.
if ! grep -q 'fn require_batch_admin' "$BATCH_FILE"; then
  echo "Batch guard is missing the admin gate for privileged operations (MintToken)" >&2
  exit 1
fi

if ! grep -q 'require_batch_admin(env)' "$BATCH_FILE"; then
  echo "Batch authorization helper does not invoke the MintToken admin gate" >&2
  exit 1
fi

if ! grep -q 'admin.require_auth();' "$BATCH_FILE"; then
  echo "Batch admin gate is missing admin authentication enforcement" >&2
  exit 1
fi

if ! grep -q 'crate::admin::require_admin(env, &admin)' "$BATCH_FILE"; then
  echo "Batch admin gate is missing the shared require_admin check" >&2
  exit 1
fi

echo "Verified KYC guard coverage for sensitive contract entry points."
