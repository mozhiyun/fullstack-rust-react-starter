import { AuthPage } from "@refinedev/mantine";

export function LoginPage() {
  return (
    <AuthPage
      type="login"
      title="Fullstack Rust React Starter"
      formProps={{
        initialValues: {
          email: "admin@example.com",
          password: "admin12345",
        },
      }}
    />
  );
}
