use std::net::SocketAddr;

use std::path::PathBuf;

use api::{router, state::AppState, ApiDoc};
use utoipa::OpenApi;
use axum::Router;
use infra::{create_pool, run_migrations, seed};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        return run_cli(&args[1]).await;
    }

    run_server().await
}

async fn run_cli(cmd: &str) -> anyhow::Result<()> {
    match cmd {
        "openapi" => write_openapi_spec(),
        "migrate" | "seed" => {
            let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let pool = create_pool(&database_url).await?;
            match cmd {
                "migrate" => {
                    run_migrations(&pool).await?;
                    tracing::info!("migrations applied");
                }
                "seed" => {
                    run_migrations(&pool).await?;
                    seed::run(&pool).await?;
                }
                _ => unreachable!(),
            }
            Ok(())
        }
        other => anyhow::bail!("unknown command: {other}. use: migrate | seed | openapi"),
    }
}

fn write_openapi_spec() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest_dir.join("../../packages/api-client/openapi.json");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let spec = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&spec)?;
    std::fs::write(&out, json)?;
    tracing::info!("wrote OpenAPI spec to {}", out.display());
    Ok(())
}

async fn run_server() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let pool = create_pool(&database_url).await?;
    // 仅应用未执行的迁移；seed 请首次用 `just seed` 手动执行
    run_migrations(&pool).await?;

    let state = AppState::from_env(pool, jwt_secret);

    let app: Router = router(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
