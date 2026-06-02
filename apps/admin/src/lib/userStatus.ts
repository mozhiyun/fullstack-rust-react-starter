export type UserStatus = "active" | "disabled";

export function normalizeUserStatus(status: string | undefined): UserStatus {
  if (!status) return "active";
  return status.toLowerCase() === "disabled" ? "disabled" : "active";
}

export function userStatusLabel(status: string | undefined): string {
  return normalizeUserStatus(status) === "active" ? "启用" : "已冻结";
}
