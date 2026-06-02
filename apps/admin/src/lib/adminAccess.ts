import type { LoginResponse } from "@fullstack-rust-react-starter/api-client";

/** 与后端 `ADMIN_ENTRY_PERMISSION` 一致 */
export const ADMIN_ENTRY_PERMISSION = "roles:read";

export function canAccessAdmin(session: LoginResponse | null | undefined): boolean {
  return session?.permissions?.includes(ADMIN_ENTRY_PERMISSION) ?? false;
}

export function adminAccessDeniedMessage(): string {
  return "该账号无权访问管理后台，请使用管理员账号登录。";
}
