import createClient from "openapi-fetch";

import type { components, paths } from "./schema";

export type LoginRequest = components["schemas"]["LoginRequest"];
export type LoginResponse = components["schemas"]["LoginResponse"];
export type RefreshRequest = components["schemas"]["RefreshRequest"];
export type LogoutRequest = components["schemas"]["LogoutRequest"];
export type CreateUser = components["schemas"]["CreateUser"];
export type UpdateUser = components["schemas"]["UpdateUser"];
export type UserPublic = components["schemas"]["UserPublic"];
export type Role = components["schemas"]["Role"];
export type Permission = components["schemas"]["Permission"];
export type AssignRoleRequest = components["schemas"]["AssignRoleRequest"];

function apiErrorMessage(error: unknown, fallback: string): string {
  if (error && typeof error === "object" && "error" in error) {
    const msg = (error as { error?: string }).error;
    if (typeof msg === "string") return msg;
  }
  return fallback;
}

export function createApiClient(baseUrl: string, accessToken?: string) {
  const client = createClient<paths>({ baseUrl });
  const auth = accessToken ? { Authorization: `Bearer ${accessToken}` } : undefined;

  return {
    health: async () => {
      const { data, error } = await client.GET("/health");
      if (error) throw new Error(apiErrorMessage(error, "health check failed"));
      return data!;
    },
    login: async (body: LoginRequest) => {
      const { data, error } = await client.POST("/api/auth/login", { body });
      if (error) throw new Error(apiErrorMessage(error, "login failed"));
      return data!;
    },
    adminLogin: async (body: LoginRequest) => {
      const { data, error } = await client.POST("/api/auth/admin/login", { body });
      if (error) throw new Error(apiErrorMessage(error, "admin login failed"));
      return data!;
    },
    register: async (body: CreateUser) => {
      const { data, error } = await client.POST("/api/auth/register", { body });
      if (error) throw new Error(apiErrorMessage(error, "register failed"));
      return data!;
    },
    refresh: async (body: RefreshRequest) => {
      const { data, error } = await client.POST("/api/auth/refresh", { body });
      if (error) throw new Error(apiErrorMessage(error, "refresh failed"));
      return data!;
    },
    adminRefresh: async (body: RefreshRequest) => {
      const { data, error } = await client.POST("/api/auth/admin/refresh", { body });
      if (error) throw new Error(apiErrorMessage(error, "admin refresh failed"));
      return data!;
    },
    logout: async (body: LogoutRequest) => {
      const { error } = await client.POST("/api/auth/logout", { body });
      if (error) throw new Error(apiErrorMessage(error, "logout failed"));
    },
    me: async () => {
      const { data, error } = await client.GET("/api/users/me", { headers: auth });
      if (error) throw new Error(apiErrorMessage(error, "fetch profile failed"));
      return data!;
    },
    listUsers: async (params?: { limit?: number; offset?: number }) => {
      const { data, error } = await client.GET("/api/users", {
        headers: auth,
        params: { query: params },
      });
      if (error) throw new Error(apiErrorMessage(error, "list users failed"));
      return data!;
    },
    getUser: async (userId: string) => {
      const { data, error } = await client.GET("/api/users/{user_id}", {
        headers: auth,
        params: { path: { user_id: userId } },
      });
      if (error) throw new Error(apiErrorMessage(error, "get user failed"));
      return data!;
    },
    createUser: async (body: CreateUser) => {
      const { data, error } = await client.POST("/api/users", {
        headers: auth,
        body,
      });
      if (error) throw new Error(apiErrorMessage(error, "create user failed"));
      return data!;
    },
    updateUser: async (userId: string, body: UpdateUser) => {
      const { data, error } = await client.PATCH("/api/users/{user_id}", {
        headers: auth,
        params: { path: { user_id: userId } },
        body,
      });
      if (error) throw new Error(apiErrorMessage(error, "update user failed"));
      return data!;
    },
    listRoles: async () => {
      const { data, error } = await client.GET("/api/roles", { headers: auth });
      if (error) throw new Error(apiErrorMessage(error, "list roles failed"));
      return data!;
    },
    listRolePermissions: async (roleId: string) => {
      const { data, error } = await client.GET("/api/roles/{role_id}/permissions", {
        headers: auth,
        params: { path: { role_id: roleId } },
      });
      if (error) throw new Error(apiErrorMessage(error, "list role permissions failed"));
      return data!;
    },
    listPermissions: async () => {
      const { data, error } = await client.GET("/api/permissions", { headers: auth });
      if (error) throw new Error(apiErrorMessage(error, "list permissions failed"));
      return data!;
    },
    listUserRoles: async (userId: string) => {
      const { data, error } = await client.GET("/api/users/{user_id}/roles", {
        headers: auth,
        params: { path: { user_id: userId } },
      });
      if (error) throw new Error(apiErrorMessage(error, "list user roles failed"));
      return data!;
    },
    listUserPermissions: async (userId: string) => {
      const { data, error } = await client.GET("/api/users/{user_id}/permissions", {
        headers: auth,
        params: { path: { user_id: userId } },
      });
      if (error) throw new Error(apiErrorMessage(error, "list user permissions failed"));
      return data!;
    },
    assignUserRole: async (userId: string, body: AssignRoleRequest) => {
      const { data, error } = await client.POST("/api/users/{user_id}/roles", {
        headers: auth,
        params: { path: { user_id: userId } },
        body,
      });
      if (error) throw new Error(apiErrorMessage(error, "assign role failed"));
      return data!;
    },
    removeUserRole: async (userId: string, roleId: string) => {
      const { error } = await client.DELETE("/api/users/{user_id}/roles/{role_id}", {
        headers: auth,
        params: { path: { user_id: userId, role_id: roleId } },
      });
      if (error) throw new Error(apiErrorMessage(error, "remove role failed"));
    },
  };
}

export type RefreshSessionFn = (
  api: ReturnType<typeof createApiClient>,
  refreshToken: string,
) => Promise<LoginResponse>;

/** 401 时用 refresh_token 换新令牌并重试一次 */
export async function withAutoRefresh<T>(
  baseUrl: string,
  session: LoginResponse,
  onSession: (next: LoginResponse) => void,
  fn: (accessToken: string) => Promise<T>,
  options?: { refreshSession?: RefreshSessionFn },
): Promise<T> {
  const api = createApiClient(baseUrl, session.access_token);
  const refreshSession =
    options?.refreshSession ??
    ((client, refreshToken) => client.refresh({ refresh_token: refreshToken }));

  try {
    return await fn(session.access_token);
  } catch (e) {
    const msg = e instanceof Error ? e.message : "";
    if (!session.refresh_token || !msg.toLowerCase().includes("unauthorized")) {
      throw e;
    }
    const next = await refreshSession(api, session.refresh_token);
    onSession(next);
    return fn(next.access_token);
  }
}
