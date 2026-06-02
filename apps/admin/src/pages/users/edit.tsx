import { Edit, useForm } from "@refinedev/mantine";
import type { UpdateUser, UserPublic } from "@fullstack-rust-react-starter/api-client";
import { Alert, Loader, Select, Stack, Text, TextInput } from "@mantine/core";
import { useParams } from "react-router";

import { UserStatusActions } from "../../components/UserStatusActions";
import { userStatusLabel } from "../../lib/userStatus";

function recordFromQuery(queryData: unknown): UserPublic | undefined {
  if (!queryData || typeof queryData !== "object") return undefined;
  const wrapped = queryData as { data?: UserPublic };
  if (wrapped.data && typeof wrapped.data === "object" && "id" in wrapped.data) {
    return wrapped.data;
  }
  if ("id" in queryData) {
    return queryData as UserPublic;
  }
  return undefined;
}

export function UserEditPage() {
  const { id } = useParams<{ id: string }>();

  const {
    saveButtonProps,
    refineCore: { formLoading, query },
    getInputProps,
  } = useForm<UserPublic, unknown, UpdateUser>({
    refineCoreProps: {
      resource: "users",
      action: "edit",
      id,
    },
    initialValues: {
      display_name: "",
      status: "active",
    },
    validate: {
      display_name: (v) => (v && v.trim().length > 0 ? null : "请输入显示名"),
    },
  });

  const user = recordFromQuery(query?.data);
  const queryPending = query?.isPending ?? query?.isLoading;
  const queryError = query?.isError ? (query.error as Error | undefined) : undefined;

  if (!id) {
    return <Alert color="red">缺少用户 ID</Alert>;
  }

  if (queryPending && !user) {
    return <Loader />;
  }

  if (queryError) {
    return (
      <Alert color="red" title="加载失败">
        {queryError.message ?? "无法获取用户信息"}
      </Alert>
    );
  }

  return (
    <Edit
      title={user?.display_name ?? "编辑用户"}
      saveButtonProps={saveButtonProps}
      isLoading={formLoading}
      headerButtons={
        user ? (
          <UserStatusActions user={user} size="sm" onUpdated={() => void query?.refetch()} />
        ) : undefined
      }
    >
      <Stack spacing="md" maw={480}>
        <TextInput label="邮箱" value={user?.email ?? ""} disabled />
        <TextInput label="显示名" required {...getInputProps("display_name")} />
        <Select
          label="状态"
          data={[
            { value: "active", label: "启用" },
            { value: "disabled", label: "已冻结" },
          ]}
          {...getInputProps("status")}
        />
        {user ? (
          <Text color="dimmed" size="sm">
            当前状态：{userStatusLabel(user.status)}。也可使用右上角「冻结 / 解冻」快速操作。
          </Text>
        ) : null}
      </Stack>
    </Edit>
  );
}
