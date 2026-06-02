import {
  useGetIdentity,
  useLogout,
  useTranslate,
  useWarnAboutChange,
} from "@refinedev/core";
import { HamburgerMenu } from "@refinedev/mantine";
import {
  Avatar,
  Flex,
  Group,
  Header,
  Menu,
  Text,
  UnstyledButton,
  useMantineTheme,
} from "@mantine/core";
import { IconChevronDown, IconLogout } from "@tabler/icons-react";

export function AdminHeader() {
  const theme = useMantineTheme();
  const translate = useTranslate();
  const { data: user } = useGetIdentity();
  const { mutate: logout } = useLogout();
  const { warnWhen, setWarnWhen } = useWarnAboutChange();

  const borderColor =
    theme.colorScheme === "dark" ? theme.colors.dark[6] : theme.colors.gray[2];

  const handleLogout = () => {
    if (warnWhen) {
      const confirm = window.confirm(
        translate(
          "warnWhenUnsavedChanges",
          "确定离开吗？你有未保存的更改。",
        ),
      );
      if (!confirm) return;
      setWarnWhen(false);
    }
    logout();
  };

  const displayName = user?.name ?? "用户";
  const email = user?.email ?? "";

  return (
    <Header
      height={64}
      py={6}
      px="sm"
      sx={{
        borderBottom: `1px solid ${borderColor}`,
        position: "sticky",
        top: 0,
        zIndex: 199,
      }}
    >
      <Flex align="center" justify="space-between" sx={{ height: "100%" }}>
        <HamburgerMenu />
        <Menu shadow="md" width={220} position="bottom-end" withinPortal>
          <Menu.Target>
            <UnstyledButton
              sx={(t) => ({
                padding: "6px 10px",
                borderRadius: t.radius.sm,
                "&:hover": {
                  backgroundColor:
                    t.colorScheme === "dark"
                      ? t.colors.dark[6]
                      : t.colors.gray[0],
                },
              })}
            >
              <Group spacing="sm">
                <Avatar src={user?.avatar} alt={displayName} radius="xl" size={32}>
                  {displayName.charAt(0).toUpperCase()}
                </Avatar>
                <div style={{ flex: 1, textAlign: "left" }}>
                  <Text size="sm" weight={500} lineClamp={1}>
                    {displayName}
                  </Text>
                  {email ? (
                    <Text size="xs" color="dimmed" lineClamp={1}>
                      {email}
                    </Text>
                  ) : null}
                </div>
                <IconChevronDown size={16} stroke={1.5} />
              </Group>
            </UnstyledButton>
          </Menu.Target>
          <Menu.Dropdown>
            {email ? <Menu.Label>{email}</Menu.Label> : null}
            <Menu.Item
              icon={<IconLogout size={16} />}
              color="red"
              onClick={handleLogout}
            >
              {translate("buttons.logout", "退出登录")}
            </Menu.Item>
          </Menu.Dropdown>
        </Menu>
      </Flex>
    </Header>
  );
}
