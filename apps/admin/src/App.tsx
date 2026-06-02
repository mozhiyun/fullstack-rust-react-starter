import { Authenticated, Refine } from "@refinedev/core";
import {
  notificationProvider,
  RefineThemes,
  ThemedLayout,
  ThemedTitle,
} from "@refinedev/mantine";
import { MantineProvider } from "@mantine/core";
import { NotificationsProvider } from "@mantine/notifications";
import routerBindings, {
  CatchAllNavigate,
  DocumentTitleHandler,
  NavigateToResource,
  UnsavedChangesNotifier,
} from "@refinedev/react-router";
import {
  IconKey,
  IconShield,
  IconUsers,
  IconUsersGroup,
} from "@tabler/icons-react";
import { BrowserRouter, Outlet, Route, Routes } from "react-router";

import { AdminHeader } from "./components/layout/AdminHeader";
import { AdminSider } from "./components/layout/AdminSider";
import { accessControlProvider } from "./providers/accessControlProvider";
import { authProvider } from "./providers/authProvider";
import { dataProvider } from "./providers/dataProvider";
import { LoginPage } from "./pages/login";
import { PermissionListPage } from "./pages/permissions/list";
import { RoleListPage } from "./pages/roles/list";
import { UserCreatePage } from "./pages/users/create";
import { UserEditPage } from "./pages/users/edit";
import { UserListPage } from "./pages/users/list";
import { UserShowPage } from "./pages/users/show";

function AppLayout() {
  return (
    <ThemedLayout
      Header={AdminHeader}
      Sider={AdminSider}
      Title={({ collapsed }) => (
        <ThemedTitle collapsed={collapsed} text="Fullstack Rust React Starter" />
      )}
    >
      <Outlet />
    </ThemedLayout>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <MantineProvider theme={RefineThemes.Blue} withGlobalStyles withNormalizeCSS>
        <NotificationsProvider position="top-right">
          <Refine
            authProvider={authProvider}
            dataProvider={dataProvider}
            accessControlProvider={accessControlProvider}
            routerProvider={routerBindings}
            notificationProvider={notificationProvider}
            resources={[
              {
                name: "user-management",
                meta: {
                  label: "用户管理",
                  icon: <IconUsersGroup size={18} />,
                },
              },
              {
                name: "users",
                list: "/users",
                create: "/users/create",
                edit: "/users/edit/:id",
                show: "/users/show/:id",
                meta: {
                  label: "用户",
                  icon: <IconUsers size={18} />,
                  parent: "user-management",
                },
              },
              {
                name: "roles",
                list: "/roles",
                meta: {
                  label: "角色",
                  icon: <IconShield size={18} />,
                  parent: "user-management",
                },
              },
              {
                name: "permissions",
                list: "/permissions",
                meta: {
                  label: "权限",
                  icon: <IconKey size={18} />,
                  parent: "user-management",
                },
              },
            ]}
            options={{
              syncWithLocation: true,
              warnWhenUnsavedChanges: true,
            }}
          >
            <Routes>
              <Route
                element={
                  <Authenticated
                    key="authenticated-inner"
                    fallback={<CatchAllNavigate to="/login" />}
                  >
                    <AppLayout />
                  </Authenticated>
                }
              >
                <Route
                  index
                  element={<NavigateToResource resource="users" />}
                />
                <Route path="users">
                  <Route index element={<UserListPage />} />
                  <Route path="create" element={<UserCreatePage />} />
                  <Route path="edit/:id" element={<UserEditPage />} />
                  <Route path="show/:id" element={<UserShowPage />} />
                </Route>
                <Route path="roles">
                  <Route index element={<RoleListPage />} />
                </Route>
                <Route path="permissions">
                  <Route index element={<PermissionListPage />} />
                </Route>
              </Route>
              <Route
                element={
                  <Authenticated
                    key="authenticated-outer"
                    fallback={<Outlet />}
                  >
                    <CatchAllNavigate to="/" />
                  </Authenticated>
                }
              >
                <Route path="login" element={<LoginPage />} />
              </Route>
            </Routes>
            <UnsavedChangesNotifier />
            <DocumentTitleHandler />
          </Refine>
        </NotificationsProvider>
      </MantineProvider>
    </BrowserRouter>
  );
}
