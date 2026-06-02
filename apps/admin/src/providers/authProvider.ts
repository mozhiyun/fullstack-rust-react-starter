import type { AuthProvider } from "@refinedev/core";

import { adminAccessDeniedMessage, canAccessAdmin } from "../lib/adminAccess";
import { createApiClient } from "../lib/api";
import { API_BASE } from "../lib/constants";
import { clearSession, getSession, setSession } from "../lib/session";

function loginErrorMessage(err: unknown): string {
  const msg = err instanceof Error ? err.message : "";
  if (msg.includes("no permission to access admin panel") || msg.includes("admin login failed")) {
    return adminAccessDeniedMessage();
  }
  if (msg.includes("account disabled")) {
    return "账号已禁用，无法登录。";
  }
  if (msg.includes("invalid credentials") || msg.includes("login failed")) {
    return "邮箱或密码错误。";
  }
  return msg || "登录失败";
}

export const authProvider: AuthProvider = {
  login: async ({ email, username, password }) => {
    const api = createApiClient(API_BASE);
    try {
      const session = await api.adminLogin({
        email: String(email ?? username),
        password: String(password),
      });
      if (!canAccessAdmin(session)) {
        return {
          success: false,
          error: { name: "Forbidden", message: adminAccessDeniedMessage() },
        };
      }
      setSession(session);
      return { success: true, redirectTo: "/" };
    } catch (e) {
      return {
        success: false,
        error: { name: "LoginError", message: loginErrorMessage(e) },
      };
    }
  },

  logout: async () => {
    const session = getSession();
    if (session?.refresh_token) {
      try {
        await createApiClient(API_BASE).logout({
          refresh_token: session.refresh_token,
        });
      } catch {
        /* ignore */
      }
    }
    clearSession();
    return { success: true, redirectTo: "/login" };
  },

  check: async () => {
    const session = getSession();
    if (!session?.access_token) {
      return { authenticated: false, redirectTo: "/login" };
    }
    if (!canAccessAdmin(session)) {
      clearSession();
      return { authenticated: false, redirectTo: "/login" };
    }
    return { authenticated: true };
  },

  getPermissions: async () => {
    const session = getSession();
    return session?.permissions ?? [];
  },

  getIdentity: async () => {
    const session = getSession();
    if (!session) return null;
    return {
      id: session.user_id,
      name: session.display_name,
      email: session.email,
    };
  },

  onError: async (error) => {
    if (error?.statusCode === 401 || error?.statusCode === 403) {
      clearSession();
      return { logout: true, redirectTo: "/login" };
    }
    return {};
  },
};
