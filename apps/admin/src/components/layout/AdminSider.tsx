import { ThemedSider } from "@refinedev/mantine";
import type { RefineThemedLayoutSiderProps } from "@refinedev/mantine";

/** 侧边栏仅保留导航；登出放在 Header 用户菜单。 */
export function AdminSider(props: RefineThemedLayoutSiderProps) {
  return <ThemedSider {...props} render={({ items }) => items} />;
}
