import {
  createApiClient,
  type LoginResponse,
  withAutoRefresh,
} from "@fullstack-rust-react-starter/api-client";

import { API_BASE } from "./constants";
import { getSession, setSession } from "./session";

export function getAuthenticatedClient() {
  const session = getSession();
  if (!session?.access_token) {
    throw new Error("未登录");
  }
  return createApiClient(API_BASE, session.access_token);
}

export async function withAuth<T>(
  fn: (accessToken: string) => Promise<T>,
): Promise<T> {
  const session = getSession();
  if (!session?.access_token) {
    throw new Error("未登录");
  }
  return withAutoRefresh(API_BASE, session, setSession, fn, {
    refreshSession: (api, refreshToken) =>
      api.adminRefresh({ refresh_token: refreshToken }),
  });
}

export { createApiClient, API_BASE };
export type { LoginResponse };
