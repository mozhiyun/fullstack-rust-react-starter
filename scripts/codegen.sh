#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> export OpenAPI from Rust (utoipa)"
cargo run -p backend -- openapi

echo "==> generate TypeScript types (openapi-typescript)"
npm run codegen -w @fullstack-rust-react-starter/api-client

echo "==> done: packages/api-client/src/schema.ts"
