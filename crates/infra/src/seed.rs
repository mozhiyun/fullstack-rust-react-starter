use anyhow::Context;
use domain::user::CreateUser;
use sqlx::PgPool;

use crate::repos::{permission_repo, role_repo, user_repo};

const ADMIN_EMAIL: &str = "admin@example.com";
const ADMIN_PASSWORD: &str = "admin12345";

struct SeedRole {
    code: &'static str,
    name: &'static str,
    description: &'static str,
}

struct SeedPermission {
    code: &'static str,
    name: &'static str,
    description: &'static str,
}

pub async fn run(pool: &PgPool) -> anyhow::Result<()> {
    let roles = [
        SeedRole {
            code: "admin",
            name: "管理员",
            description: "系统管理员，拥有全部权限",
        },
        SeedRole {
            code: "user",
            name: "普通用户",
            description: "默认注册用户角色",
        },
    ];

    let permissions = [
        SeedPermission {
            code: "users:read",
            name: "查看用户",
            description: "列出与查看用户",
        },
        SeedPermission {
            code: "users:write",
            name: "管理用户",
            description: "创建、禁用用户",
        },
        SeedPermission {
            code: "roles:read",
            name: "查看角色",
            description: "查看角色与权限配置",
        },
        SeedPermission {
            code: "roles:write",
            name: "管理角色",
            description: "分配角色与权限",
        },
    ];

    for r in roles {
        sqlx::query(
            r#"
            INSERT INTO roles (code, name, description)
            VALUES ($1, $2, $3)
            ON CONFLICT (code) DO NOTHING
            "#,
        )
        .bind(r.code)
        .bind(r.name)
        .bind(r.description)
        .execute(pool)
        .await
        .context("seed roles")?;
    }

    for p in &permissions {
        sqlx::query(
            r#"
            INSERT INTO permissions (code, name, description)
            VALUES ($1, $2, $3)
            ON CONFLICT (code) DO NOTHING
            "#,
        )
        .bind(p.code)
        .bind(p.name)
        .bind(p.description)
        .execute(pool)
        .await
        .context("seed permissions")?;
    }

    let admin_role = role_repo::find_by_code(pool, "admin").await?;
    let user_role = role_repo::find_by_code(pool, "user").await?;

    for p in permissions {
        let perm = permission_repo::find_by_code(pool, p.code).await?;
        permission_repo::assign_to_role(pool, admin_role.id, perm.id).await?;
    }

    let user_read = permission_repo::find_by_code(pool, "users:read").await?;
    permission_repo::assign_to_role(pool, user_role.id, user_read.id).await?;

    if user_repo::find_by_email(pool, ADMIN_EMAIL).await?.is_none() {
        let admin = user_repo::create(
            pool,
            CreateUser {
                email: ADMIN_EMAIL.into(),
                password: ADMIN_PASSWORD.into(),
                display_name: "系统管理员".into(),
            },
        )
        .await
        .context("create admin user")?;

        role_repo::assign_to_user(pool, admin.id, admin_role.id).await?;
    }

    println!(
        "seed complete — admin login: {ADMIN_EMAIL} / {ADMIN_PASSWORD}"
    );

    Ok(())
}
