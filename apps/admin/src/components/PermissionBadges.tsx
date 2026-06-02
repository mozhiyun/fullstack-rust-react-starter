import type { Permission } from "@fullstack-rust-react-starter/api-client";
import { Badge, Group, Text } from "@mantine/core";

type Props = {
  permissions: Permission[];
  emptyLabel?: string;
};

export function PermissionBadges({ permissions, emptyLabel = "无" }: Props) {
  if (permissions.length === 0) {
    return (
      <Text color="dimmed" size="sm">
        {emptyLabel}
      </Text>
    );
  }

  return (
    <Group spacing={6}>
      {permissions.map((p) => (
        <Badge key={p.id} variant="outline" title={p.name}>
          {p.code}
        </Badge>
      ))}
    </Group>
  );
}
