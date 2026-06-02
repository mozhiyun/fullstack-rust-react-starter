import type { BaseRecord, DataProvider } from "@refinedev/core";
import type { CreateUser, UpdateUser } from "@fullstack-rust-react-starter/api-client";

import { getAuthenticatedClient, withAuth } from "../lib/api";

export const dataProvider = {
  getList: async ({ resource, pagination }) => {
    const pageSize = pagination?.pageSize ?? 20;
    const current = pagination?.currentPage ?? 1;
    const offset = (current - 1) * pageSize;

    return withAuth(async () => {
      const api = getAuthenticatedClient();

      if (resource === "users") {
        const data = await api.listUsers({ limit: pageSize, offset });
        const total =
          data.length < pageSize ? offset + data.length : offset + pageSize + 1;
        return { data: data as BaseRecord[], total };
      }
      if (resource === "roles") {
        const data = await api.listRoles();
        return { data: data as BaseRecord[], total: data.length };
      }
      if (resource === "permissions") {
        const data = await api.listPermissions();
        return { data: data as BaseRecord[], total: data.length };
      }

      throw new Error(`Unknown resource: ${resource}`);
    });
  },

  getOne: async ({ resource, id }) => {
    return withAuth(async () => {
      const api = getAuthenticatedClient();

      if (resource === "users") {
        const record = await api.getUser(String(id));
        return { data: record as BaseRecord };
      }

      throw new Error(`getOne not supported for ${resource}`);
    });
  },

  create: async ({ resource, variables }) => {
    return withAuth(async () => {
      const api = getAuthenticatedClient();

      if (resource === "users") {
        const data = await api.createUser(variables as CreateUser);
        return { data: data as BaseRecord };
      }

      throw new Error(`create not supported for ${resource}`);
    });
  },

  update: async ({ resource, id, variables }) => {
    return withAuth(async () => {
      const api = getAuthenticatedClient();

      if (resource === "users") {
        const data = await api.updateUser(String(id), variables as UpdateUser);
        return { data: data as BaseRecord };
      }

      throw new Error(`update not supported for ${resource}`);
    });
  },

  deleteOne: async () => {
    throw new Error("deleteOne not implemented");
  },

  getApiUrl: () => "",
} as DataProvider;
