# 本地开发入口（方案 A：npm workspaces + just + Cargo）

set dotenv-load := true

compose_file := "docker-compose.yml"
compose_example := "docker-compose.example.yml"

# 从 example 复制 compose（不存在时）
init-compose:
    @test -f {{compose_file}} || cp {{compose_example}} {{compose_file}}

# 启动 Postgres + Redis（dev / dev-api 不会自动执行）
dev-db: init-compose
    docker compose up -d

# 停止 Postgres + Redis
stop-db: init-compose
    docker compose down

# 导出 OpenAPI 并生成 @app/api-client 类型
codegen:
    bash scripts/codegen.sh

# 应用未执行的数据库迁移（表结构变更后执行）
migrate:
    cargo run -p backend -- migrate

# 种子数据（角色/权限/管理员），新库只需执行一次
seed:
    cargo run -p backend -- seed

# 后端 API（需先 just dev-db；启动时自动 migrate，不自动 seed）
dev-api:
    cargo run -p backend

# ToC 前端
dev-web:
    npm run dev -w web

# 管理后台
dev-admin:
    npm run dev -w admin

# 并行启动 API + 两个前端（需自行确保 Postgres 已就绪）
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill 0' EXIT INT TERM
    cargo run -p backend &
    npm run dev -w web &
    npm run dev -w admin &
    wait

# 构建全部
build:
    cargo build --workspace
    npm run build -ws

# 测试
test:
    cargo test --workspace
