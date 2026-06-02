import { useList } from "@refinedev/core";
import { List } from "@refinedev/mantine";
import type { Permission, Role } from "@fullstack-rust-react-starter/api-client";
import { Loader, Table, Text } from "@mantine/core";
import { useEffect, useState } from "react";

import { PermissionBadges } from "../../components/PermissionBadges";
import { getAuthenticatedClient, withAuth } from "../../lib/api";

export function RoleListPage() {
  const { result, query } = useList<Role>({ resource: "roles" });
  const roles = result?.data ?? [];
  const isLoading = query.isLoading;

  const [permsByRoleId, setPermsByRoleId] = useState<Record<string, Permission[]>>({});
  const [permsLoading, setPermsLoading] = useState(false);

  useEffect(() => {
    if (roles.length === 0) {
      setPermsByRoleId({});
      return;
    }

    let cancelled = false;
    setPermsLoading(true);

    withAuth(async () => {
      const api = getAuthenticatedClient();
      const entries = await Promise.all(
        roles.map(async (role) => {
          const perms = await api.listRolePermissions(role.id);
          return [role.id, perms] as const;
        }),
      );
      if (!cancelled) {
        setPermsByRoleId(Object.fromEntries(entries));
      }
    })
      .catch(() => {
        if (!cancelled) setPermsByRoleId({});
      })
      .finally(() => {
        if (!cancelled) setPermsLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [roles]);

  return (
    <List title="角色">
      <Text color="dimmed" size="sm" mb="md">
        下表展示每个角色在系统中绑定的权限。例如「普通用户」角色默认仅有{" "}
        <code>users:read</code>（见种子数据 <code>just seed</code>）。
      </Text>
      <Table highlightOnHover>
        <thead>
          <tr>
            <th>Code</th>
            <th>名称</th>
            <th>说明</th>
            <th>所含权限</th>
          </tr>
        </thead>
        <tbody>
          {isLoading ? (
            <tr>
              <td colSpan={4}>
                <Loader size="sm" />
              </td>
            </tr>
          ) : (
            roles.map((role) => (
              <tr key={role.id}>
                <td>{role.code}</td>
                <td>{role.name}</td>
                <td>{role.description ?? "—"}</td>
                <td>
                  {permsLoading ? (
                    <Loader size="xs" />
                  ) : (
                    <PermissionBadges
                      permissions={permsByRoleId[role.id] ?? []}
                      emptyLabel="未绑定权限"
                    />
                  )}
                </td>
              </tr>
            ))
          )}
        </tbody>
      </Table>
    </List>
  );
}
