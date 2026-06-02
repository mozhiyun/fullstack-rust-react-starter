# fullstack-rust-react-starter

方案 A 单仓库模板：**npm workspaces + just + Cargo workspace + PostgreSQL**。

## 结构

```text
apps/
  backend/     # API 入口：serve | migrate | seed | openapi
  web/         # ToC 前端 (5173)
  admin/       # 管理后台 Refine + Mantine (5174)
crates/
  domain/      # 实体与 DTO（utoipa ToSchema）
  infra/       # 数据库、迁移、仓储、种子数据
  api/         # Axum 路由、JWT、权限中间件、OpenAPI
packages/
  api-client/  # 由 OpenAPI 生成的 TS 类型 + fetch 封装
```

## 数据模型（RBAC）

| 表 | 说明 |
|----|------|
| `users` | 用户（邮箱、密码哈希、状态） |
| `roles` | 角色（`admin`、`user`） |
| `permissions` | 权限（`users:read` 等） |
| `user_roles` | 用户 ↔ 角色 |
| `role_permissions` | 角色 ↔ 权限 |

种子管理员：`admin@example.com` / `admin12345`（上线前务必修改）。

## HTTP 中间件

全局层（见 `crates/api/src/middleware/mod.rs`）：

| 能力 | 说明 |
|------|------|
| Request ID | `X-Request-Id` 响应头 + 日志字段 |
| 访问日志 | 每请求一行，含 `user_id`（已登录时） |
| 限流 | `/api/auth/*` 默认 20 次/分钟/IP，其它 300 次/分钟/IP |
| Body 限制 | 默认 2MB（`HTTP_BODY_LIMIT_MB`） |
| 安全响应头 | `nosniff`、`X-Frame-Options: DENY`、`Referrer-Policy` |
| RBAC 缓存 | 用户权限/状态 TTL 默认 60s，改角色或用户状态会失效 |

## HTTP 访问日志

每个请求一行（`.env` 中 `RUST_LOG=info`）。按 HTTP 状态选级别：2xx/3xx → INFO，4xx → WARN，5xx → ERROR。示例：

```text
INFO api::middleware::request_context: http request request_id=... method=GET uri=/health status=200 latency_ms=1
WARN api::middleware::request_context: http request ... status=401 ...
```

**修改后请重启 `just dev-api`**。

## 路由级权限（无 Casbin）

受保护路由在 `crates/api/src/routes/mod.rs` 通过 `route_with_permission` 声明所需权限，由 `require_permission` 中间件查 RBAC 表：

| 路由 | 权限 |
|------|------|
| `GET /api/users` | `users:read` |
| `GET /api/roles` | `roles:read` |
| `GET /api/permissions` | `roles:read` |
| `GET /api/users/me` | 仅需登录 |

## OpenAPI 与 api-client

- 运行时文档：<http://localhost:8080/swagger-ui>
- 规范 JSON：<http://localhost:8080/api/openapi.json>
- 改 Rust DTO / `#[utoipa::path]` 后重新生成前端类型：

```bash
just codegen
# 或
npm run codegen
```

流程：`cargo run -p backend -- openapi` → `openapi-typescript` → `packages/api-client/src/schema.ts`。

前端统一使用 `@fullstack-rust-react-starter/api-client` 的 `createApiClient()`（`fsr new` 后会变为 `@<你的项目名>/api-client`），**不要手改** `schema.ts`。

## 管理后台（Refine + Mantine）

- 与 `web` 的 React 19 隔离：admin 固定 React 18，`vite.config.ts` 通过 alias 指向仓库根 `node_modules/react`
- 登录、侧边栏布局、用户/角色/权限列表
- RBAC 菜单与按钮按 `permissions` 控制（如无 `users:read` 则看不到用户）
- 用户详情页：分配 / 移除角色（需 `roles:write`）
- 默认账号：`admin@example.com` / `admin12345`（须具备 `roles:read` 等管理权限；`user` 角色仅用于 ToC，不能登录管理后台）

## 模板占位符（`fsr new` 会替换）

| 占位 | 示例替换为 `my-app` | 出现位置 |
|------|---------------------|----------|
| `fullstack-rust-react-starter` | `my-app` | 根 `package.json`、`@…/api-client` scope、Docker 容器名 |
| `fullstack_rust_react_starter` | `my_app` | `POSTGRES_DB`、`DATABASE_URL`、Docker volume |
| `Fullstack Rust React Starter` | `My App` | 页面标题、侧栏、登录页 |
| `Fullstack Rust React Starter API` | `My App API` | OpenAPI `info.title` |

工作区包名 `web` / `admin` / `backend` 等**保持不变**，便于 `just dev-*` 与 npm `-w` 脚本。

## 用 CLI 创建新项目（`fsr`）

CLI 在**独立仓库** [mozhiyun/fsr](https://github.com/mozhiyun/fsr)（本仓库仅为模板，不含 CLI 源码）：

```bash
cargo install --git https://github.com/mozhiyun/fsr
# 或克隆：git clone git@github.com:mozhiyun/fsr.git
fsr new my-app
cd my-app && npm install
```

`fsr` 默认克隆 [mozhiyun/fullstack-rust-react-starter](https://github.com/mozhiyun/fullstack-rust-react-starter)。本地开发请用：

```bash
fsr new demo-app --template-dir /path/to/fullstack-rust-react-starter
```

（`FSR_TEMPLATE_REPO=...` 须与 `fsr` 写在**同一行**。）详见 fsr 仓库 README。

## 快速开始（本仓库内开发）

```bash
cp .env.example .env
cp docker-compose.example.yml docker-compose.yml
npm install
just dev-db
just migrate && just seed   # 新库只做一次；seed 可重复执行（幂等）
just codegen                # 首次或 API 变更后
just dev-api                # :8080（之后启动会自动 migrate，不自动 seed）
just dev-web                # :5173
just dev-admin              # :5174
```

**日常开发**：`just dev-db`（库未起时）→ `just dev` 或分别起 api/web/admin。  
**表结构改了**（新增 `crates/infra/migrations/*.sql`）→ 再跑 `just migrate`（或只重启 `dev-api` 也会自动 migrate）。

## API 摘要

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 健康检查 |
| POST | `/api/auth/register` | 注册 |
| POST | `/api/auth/login` | 登录（ToC / 通用） |
| POST | `/api/auth/admin/login` | 管理后台登录（须 `roles:read`） |
| POST | `/api/auth/admin/refresh` | 管理后台刷新令牌（同上） |
| GET | `/api/users/me` | 当前用户（Bearer） |
| GET | `/api/users` | 用户列表（`users:read`） |
| GET | `/api/roles` | 角色列表（`roles:read`） |
| GET | `/api/permissions` | 权限列表（`roles:read`） |
| POST | `/api/auth/refresh` | 用 refresh_token 换新令牌对 |
| POST | `/api/auth/logout` | 撤销 refresh_token |
| GET | `/api/users/{id}/roles` | 用户的角色（`roles:read`） |
| POST | `/api/users/{id}/roles` | 分配角色（`roles:write`） |
| DELETE | `/api/users/{id}/roles/{role_id}` | 移除角色（`roles:write`） |

登录/注册返回 `access_token`（默认 15 分钟）+ `refresh_token`（默认 7 天，存库可撤销）。

## 环境变量

见 [.env.example](.env.example)：`POSTGRES_PORT`、`REDIS_PORT`、`DATABASE_URL`、`REDIS_URL`、`JWT_SECRET` 等。

Docker 使用 [docker-compose.example.yml](docker-compose.example.yml)，复制为 `docker-compose.yml`（已 gitignore）。`just dev-db` 会读 `.env` 里的端口并启动 **PostgreSQL** 与 **Redis**；若改 `POSTGRES_PORT` / `REDIS_PORT`，须同步修改 `DATABASE_URL` / `REDIS_URL`。
