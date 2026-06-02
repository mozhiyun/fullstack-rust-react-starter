import type { AccessControlProvider } from "@refinedev/core";

import { ADMIN_ENTRY_PERMISSION } from "../lib/adminAccess";
import { getSession } from "../lib/session";

function hasPermission(code: string): boolean {
  const session = getSession();
  return session?.permissions?.includes(code) ?? false;
}

export const accessControlProvider: AccessControlProvider = {
  can: async ({ resource, action }) => {
    if (resource === "users") {
      if (action === "list" || action === "show") {
        return { can: hasPermission("users:read") };
      }
      if (action === "edit" || action === "create") {
        return { can: hasPermission("users:write") };
      }
    }
    if (resource === "roles") {
      if (action === "edit") {
        return { can: hasPermission("roles:write") };
      }
      return { can: hasPermission("roles:read") };
    }
    if (resource === "permissions") {
      return { can: hasPermission("roles:read") };
    }
    return { can: hasPermission(ADMIN_ENTRY_PERMISSION) };
  },
};
