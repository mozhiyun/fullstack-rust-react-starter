import { useList } from "@refinedev/core";
import { List } from "@refinedev/mantine";
import type { Permission } from "@fullstack-rust-react-starter/api-client";
import { Table, Text } from "@mantine/core";

export function PermissionListPage() {
  const { result, query } = useList<Permission>({ resource: "permissions" });
  const rows = result?.data ?? [];
  const isLoading = query.isLoading;

  return (
    <List title="权限">
      <Text color="dimmed" size="sm" mb="md">
        系统内全部权限定义（资源:操作）。某角色实际拥有哪些权限，请到「角色」页查看；某用户有效权限，请到用户详情查看。
      </Text>
      <Table highlightOnHover>
        <thead>
          <tr>
            <th>Code</th>
            <th>名称</th>
            <th>说明</th>
          </tr>
        </thead>
        <tbody>
          {isLoading ? (
            <tr>
              <td colSpan={3}>加载中…</td>
            </tr>
          ) : (
            rows.map((p) => (
              <tr key={p.id}>
                <td>
                  <code>{p.code}</code>
                </td>
                <td>{p.name}</td>
                <td>{p.description ?? "—"}</td>
              </tr>
            ))
          )}
        </tbody>
      </Table>
    </List>
  );
}
