import { useCan, useShow } from "@refinedev/core";
import { Show } from "@refinedev/mantine";
import type { Permission, Role, UserPublic } from "@fullstack-rust-react-starter/api-client";
import {
  ActionIcon,
  Badge,
  Button,
  Group,
  Loader,
  Select,
  Stack,
  Table,
  Text,
  Title,
} from "@mantine/core";
import { showNotification } from "@mantine/notifications";
import { IconTrash } from "@tabler/icons-react";
import { useCallback, useEffect, useState } from "react";

import { PermissionBadges } from "../../components/PermissionBadges";
import { UserStatusActions } from "../../components/UserStatusActions";
import { getAuthenticatedClient, withAuth } from "../../lib/api";
import { normalizeUserStatus, userStatusLabel } from "../../lib/userStatus";

export function UserShowPage() {
  const { query, result: user } = useShow<UserPublic>();
  const userId = user?.id;

  const { data: canEditUser } = useCan({ resource: "users", action: "edit" });
  const { data: canManageRoles } = useCan({ resource: "roles", action: "edit" });

  const [userRoles, setUserRoles] = useState<Role[]>([]);
  const [userPermissions, setUserPermissions] = useState<Permission[]>([]);
  const [allRoles, setAllRoles] = useState<Role[]>([]);
  const [rolesLoading, setRolesLoading] = useState(false);
  const [selectedRoleId, setSelectedRoleId] = useState<string | null>(null);

  const loadRoles = useCallback(async () => {
    if (!userId) return;
    setRolesLoading(true);
    try {
      await withAuth(async () => {
        const api = getAuthenticatedClient();
        const [ur, ar, up] = await Promise.all([
          api.listUserRoles(userId),
          api.listRoles(),
          api.listUserPermissions(userId),
        ]);
        setUserRoles(ur);
        setAllRoles(ar);
        setUserPermissions(up);
      });
    } finally {
      setRolesLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    loadRoles();
  }, [loadRoles]);

  const assignedIds = new Set(userRoles.map((r) => r.id));
  const availableRoles = allRoles.filter((r) => !assignedIds.has(r.id));

  async function handleAssign() {
    if (!userId || !selectedRoleId) return;
    try {
      await withAuth(async () => {
        const api = getAuthenticatedClient();
        await api.assignUserRole(userId, { role_id: selectedRoleId });
      });
      showNotification({ title: "已分配角色", message: "", color: "green" });
      setSelectedRoleId(null);
      await loadRoles();
    } catch (e) {
      showNotification({
        title: "分配失败",
        message: e instanceof Error ? e.message : "未知错误",
        color: "red",
      });
    }
  }

  async function handleRemove(roleId: string) {
    if (!userId) return;
    try {
      await withAuth(async () => {
        const api = getAuthenticatedClient();
        await api.removeUserRole(userId, roleId);
      });
      showNotification({ title: "已移除角色", message: "", color: "green" });
      await loadRoles();
    } catch (e) {
      showNotification({
        title: "移除失败",
        message: e instanceof Error ? e.message : "未知错误",
        color: "red",
      });
    }
  }

  if (query.isLoading || !user) {
    return <Loader />;
  }

  return (
    <Show
      title={user.display_name}
      canEdit={canEditUser?.can}
      headerButtons={
        <UserStatusActions
          user={user}
          size="sm"
          onUpdated={() => {
            query.refetch();
            loadRoles();
          }}
        />
      }
    >
      <Stack spacing="lg">
        <Group>
          <Text weight={500}>邮箱</Text>
          <Text>{user.email}</Text>
          <Badge color={normalizeUserStatus(user.status) === "active" ? "green" : "red"}>
            {userStatusLabel(user.status)}
          </Badge>
        </Group>

        <div>
          <Title order={4} mb="sm">
            有效权限
          </Title>
          <Text color="dimmed" size="sm" mb="xs">
            由其已分配角色的权限合并去重得出。
          </Text>
          {rolesLoading ? (
            <Loader size="sm" />
          ) : (
            <PermissionBadges
              permissions={userPermissions}
              emptyLabel="暂无权限（未分配角色或角色未绑定权限）"
            />
          )}
        </div>

        <div>
          <Title order={4} mb="sm">
            已分配角色
          </Title>
          {rolesLoading ? (
            <Loader size="sm" />
          ) : (
            <Table>
              <thead>
                <tr>
                  <th>Code</th>
                  <th>名称</th>
                  {canManageRoles?.can && <th>操作</th>}
                </tr>
              </thead>
              <tbody>
                {userRoles.map((role) => (
                  <tr key={role.id}>
                    <td>{role.code}</td>
                    <td>{role.name}</td>
                    {canManageRoles?.can && (
                      <td>
                        <ActionIcon
                          color="red"
                          variant="subtle"
                          onClick={() => handleRemove(role.id)}
                        >
                          <IconTrash size={16} />
                        </ActionIcon>
                      </td>
                    )}
                  </tr>
                ))}
                {userRoles.length === 0 && (
                  <tr>
                    <td colSpan={3}>
                      <Text color="dimmed" size="sm">
                        暂无角色
                      </Text>
                    </td>
                  </tr>
                )}
              </tbody>
            </Table>
          )}
        </div>

        {canManageRoles?.can && (
          <Group>
            <Select
              placeholder="选择要分配的角色"
              data={availableRoles.map((r) => ({
                value: r.id,
                label: `${r.code} — ${r.name}`,
              }))}
              value={selectedRoleId}
              onChange={setSelectedRoleId}
              style={{ minWidth: 280 }}
              clearable
            />
            <Button onClick={handleAssign} disabled={!selectedRoleId}>
              分配角色
            </Button>
          </Group>
        )}
      </Stack>
    </Show>
  );
}
