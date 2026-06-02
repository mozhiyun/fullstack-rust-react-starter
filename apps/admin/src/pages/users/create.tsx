import { Create, useForm } from "@refinedev/mantine";
import type { CreateUser } from "@fullstack-rust-react-starter/api-client";
import { PasswordInput, Stack, TextInput } from "@mantine/core";

export function UserCreatePage() {
  const {
    saveButtonProps,
    refineCore: { formLoading },
    getInputProps,
  } = useForm<CreateUser>({
    refineCoreProps: {
      resource: "users",
      action: "create",
    },
    initialValues: {
      email: "",
      password: "",
      display_name: "",
    },
    validate: {
      email: (v) => (/^\S+@\S+\.\S+$/.test(v) ? null : "请输入有效邮箱"),
      password: (v) => (v.length >= 8 ? null : "密码至少 8 位"),
      display_name: (v) => (v.trim().length > 0 ? null : "请输入显示名"),
    },
  });

  return (
    <Create title="新建用户" saveButtonProps={saveButtonProps} isLoading={formLoading}>
      <Stack spacing="md" maw={480}>
        <TextInput
          label="邮箱"
          placeholder="user@example.com"
          required
          {...getInputProps("email")}
        />
        <TextInput
          label="显示名"
          placeholder="张三"
          required
          {...getInputProps("display_name")}
        />
        <PasswordInput
          label="密码"
          placeholder="至少 8 位"
          required
          {...getInputProps("password")}
        />
      </Stack>
    </Create>
  );
}
