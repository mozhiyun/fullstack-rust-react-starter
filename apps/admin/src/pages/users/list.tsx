import { useCan, useList, useNavigation } from "@refinedev/core";
import { CreateButton, List } from "@refinedev/mantine";
import type { UserPublic } from "@fullstack-rust-react-starter/api-client";
import { Badge, Button, Group, Pagination, Select, Table } from "@mantine/core";
import { IconEdit, IconEye } from "@tabler/icons-react";
import { useState } from "react";

import { UserStatusActions } from "../../components/UserStatusActions";
import { normalizeUserStatus, userStatusLabel } from "../../lib/userStatus";

export function UserListPage() {
  const { show, edit } = useNavigation();
  const { data: canEditUser } = useCan({ resource: "users", action: "edit" });
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  const { result, query } = useList<UserPublic>({
    resource: "users",
    pagination: { currentPage: page, pageSize, mode: "server" },
  });

  const users = (result?.data ?? []) as UserPublic[];
  const total = result?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const isLoading = query.isLoading;

  return (
    <List title="用户" headerButtons={<CreateButton />}>
      <Table highlightOnHover verticalSpacing="sm">
        <thead>
          <tr>
            <th>显示名</th>
            <th>邮箱</th>
            <th>状态</th>
            <th>注册时间</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          {isLoading ? (
            <tr>
              <td colSpan={5}>加载中…</td>
            </tr>
          ) : (
            users.map((user) => (
              <tr key={user.id}>
                <td>{user.display_name}</td>
                <td>{user.email}</td>
                <td>
                  <Badge
                    color={normalizeUserStatus(user.status) === "active" ? "green" : "red"}
                    variant={normalizeUserStatus(user.status) === "active" ? "light" : "filled"}
                  >
                    {userStatusLabel(user.status)}
                  </Badge>
                </td>
                <td>{new Date(user.created_at).toLocaleString()}</td>
                <td>
                  <Group spacing={6} noWrap>
                    <Button
                      size="xs"
                      variant="light"
                      leftIcon={<IconEye size={14} />}
                      onClick={() => show("users", user.id)}
                    >
                      详情
                    </Button>
                    {canEditUser?.can ? (
                      <>
                        <Button
                          size="xs"
                          variant="light"
                          leftIcon={<IconEdit size={14} />}
                          onClick={() => edit("users", user.id)}
                        >
                          编辑
                        </Button>
                        <UserStatusActions user={user} />
                      </>
                    ) : null}
                  </Group>
                </td>
              </tr>
            ))
          )}
        </tbody>
      </Table>
      <Group position="right" spacing="md" mt="md">
        <Select
          value={String(pageSize)}
          onChange={(v) => {
            if (!v) return;
            setPageSize(Number(v));
            setPage(1);
          }}
          data={[
            { value: "10", label: "10 / 页" },
            { value: "20", label: "20 / 页" },
            { value: "50", label: "50 / 页" },
          ]}
          w={100}
        />
        <Pagination page={page} onChange={setPage} total={totalPages} />
      </Group>
    </List>
  );
}
