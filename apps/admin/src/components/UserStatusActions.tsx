import { useCan, useInvalidate } from "@refinedev/core";
import type { UserPublic } from "@fullstack-rust-react-starter/api-client";
import { Button, Group } from "@mantine/core";
import { showNotification } from "@mantine/notifications";
import { useState } from "react";

import { getAuthenticatedClient, withAuth } from "../lib/api";
import { normalizeUserStatus } from "../lib/userStatus";

type Props = {
  user: UserPublic;
  size?: "xs" | "sm" | "md";
  onUpdated?: () => void;
};

export function UserStatusActions({ user, size = "xs", onUpdated }: Props) {
  const { data: canWrite } = useCan({ resource: "users", action: "edit" });
  const invalidate = useInvalidate();
  const [loading, setLoading] = useState(false);

  if (!canWrite?.can) {
    return null;
  }

  const isActive = normalizeUserStatus(user.status) === "active";

  async function applyStatus(status: "active" | "disabled") {
    const label = status === "disabled" ? "冻结" : "解冻";
    const ok = window.confirm(
      status === "disabled"
        ? `确定冻结用户「${user.display_name}」？冻结后将无法登录，已有令牌也会立即失效。`
        : `确定解冻用户「${user.display_name}」？`,
    );
    if (!ok) return;

    setLoading(true);
    try {
      await withAuth(async () => {
        const api = getAuthenticatedClient();
        await api.updateUser(user.id, { status });
      });
      showNotification({ title: `已${label}`, message: user.email, color: "green" });
      await invalidate({
        resource: "users",
        invalidates: ["list", "detail"],
        id: user.id,
      });
      onUpdated?.();
    } catch (e) {
      showNotification({
        title: `${label}失败`,
        message: e instanceof Error ? e.message : "未知错误",
        color: "red",
      });
    } finally {
      setLoading(false);
    }
  }

  return (
    <Group spacing={6} noWrap>
      {isActive ? (
        <Button
          size={size}
          color="red"
          variant="filled"
          loading={loading}
          onClick={() => applyStatus("disabled")}
        >
          冻结
        </Button>
      ) : (
        <Button
          size={size}
          color="teal"
          variant="filled"
          loading={loading}
          onClick={() => applyStatus("active")}
        >
          解冻
        </Button>
      )}
    </Group>
  );
}
