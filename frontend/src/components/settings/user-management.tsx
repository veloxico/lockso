import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Shield,
  ShieldOff,
  Trash2,
  Crown,
  UserCog,
  User,
  UserPlus,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Spinner } from "@/components/ui/spinner";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog";
import { userManagementApi } from "@/api/admin";
import { useAuthStore } from "@/stores/auth";
import type { AdminUserListItem, UserRoleView } from "@/types/admin";

const ROLE_ICONS: Record<string, typeof Crown> = {
  owner: Crown,
  admin: UserCog,
  user: User,
};

const ROLE_BADGE_VARIANT: Record<string, "default" | "secondary" | "outline"> = {
  owner: "default",
  admin: "secondary",
  user: "outline",
};

export function UserManagement() {
  const { t } = useTranslation();
  const currentUserId = useAuthStore((s) => s.user?.id);

  const [users, setUsers] = useState<AdminUserListItem[]>([]);
  const [roles, setRoles] = useState<UserRoleView[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Dialog state
  const [roleDialogUser, setRoleDialogUser] = useState<AdminUserListItem | null>(null);
  const [selectedRoleId, setSelectedRoleId] = useState("");
  const [deleteDialogUser, setDeleteDialogUser] = useState<AdminUserListItem | null>(null);
  const [actionLoading, setActionLoading] = useState(false);

  // Create user dialog
  const [createOpen, setCreateOpen] = useState(false);
  const [newLogin, setNewLogin] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [newEmail, setNewEmail] = useState("");
  const [newFullName, setNewFullName] = useState("");
  const [newRoleId, setNewRoleId] = useState("");

  const loadData = useCallback(async () => {
    try {
      const [usersData, rolesData] = await Promise.all([
        userManagementApi.list(),
        userManagementApi.listRoles(),
      ]);
      setUsers(usersData);
      setRoles(rolesData);
    } catch {
      setError(t("settings.errorLoadFailed"));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleToggleBlock = async (user: AdminUserListItem) => {
    try {
      await userManagementApi.setBlocked(user.id, !user.isBlocked);
      await loadData();
    } catch {
      setError(t("settings.errorActionFailed"));
    }
  };

  const handleRoleChange = async () => {
    if (!roleDialogUser || !selectedRoleId) return;
    setActionLoading(true);
    try {
      await userManagementApi.updateRole(roleDialogUser.id, selectedRoleId);
      setRoleDialogUser(null);
      await loadData();
    } catch {
      setError(t("settings.errorActionFailed"));
    } finally {
      setActionLoading(false);
    }
  };

  const handleCreateUser = async () => {
    if (!newLogin.trim() || !newPassword.trim()) return;
    setActionLoading(true);
    setError("");
    try {
      await userManagementApi.create({
        login: newLogin.trim(),
        password: newPassword,
        email: newEmail.trim() || undefined,
        fullName: newFullName.trim() || undefined,
        roleId: newRoleId || undefined,
      });
      setCreateOpen(false);
      setNewLogin("");
      setNewPassword("");
      setNewEmail("");
      setNewFullName("");
      setNewRoleId("");
      await loadData();
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err
        ? String((err as { message: string }).message)
        : t("settings.errorActionFailed");
      setError(msg);
    } finally {
      setActionLoading(false);
    }
  };

  const handleDeleteUser = async () => {
    if (!deleteDialogUser) return;
    setActionLoading(true);
    try {
      await userManagementApi.delete(deleteDialogUser.id);
      setDeleteDialogUser(null);
      await loadData();
    } catch {
      setError(t("settings.errorActionFailed"));
    } finally {
      setActionLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex justify-center py-12">
        <Spinner size="md" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">{t("settings.usersTitle")}</h2>
          <p className="text-sm text-muted-foreground">
            {t("settings.usersDescription", { count: users.length })}
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)} size="sm">
          <UserPlus className="h-4 w-4" />
          {t("settings.createUser")}
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {/* User table */}
      <div className="rounded-lg border border-border overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-border bg-muted/50">
              <th className="px-4 py-3 text-left font-medium text-muted-foreground">
                {t("settings.colUser")}
              </th>
              <th className="px-4 py-3 text-left font-medium text-muted-foreground">
                {t("settings.colRole")}
              </th>
              <th className="px-4 py-3 text-left font-medium text-muted-foreground">
                {t("settings.colStatus")}
              </th>
              <th className="px-4 py-3 text-left font-medium text-muted-foreground">
                {t("settings.colLastLogin")}
              </th>
              <th className="px-4 py-3 text-right font-medium text-muted-foreground">
                {t("settings.colActions")}
              </th>
            </tr>
          </thead>
          <tbody>
            {users.map((user) => {
              const isOwner = user.roleCode === "owner";
              const isSelf = user.id === currentUserId;
              const RoleIcon = ROLE_ICONS[user.roleCode] || User;

              return (
                <tr
                  key={user.id}
                  className="border-b border-border last:border-0 hover:bg-muted/30 transition-colors"
                >
                  {/* User info */}
                  <td className="px-4 py-3">
                    <div>
                      <p className="font-medium text-foreground">
                        {user.fullName || user.login}
                        {isSelf && (
                          <span className="ml-1.5 text-xs text-muted-foreground">
                            ({t("settings.you")})
                          </span>
                        )}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {user.login}
                        {user.email && ` · ${user.email}`}
                      </p>
                    </div>
                  </td>

                  {/* Role */}
                  <td className="px-4 py-3">
                    <Badge variant={ROLE_BADGE_VARIANT[user.roleCode] || "outline"}>
                      <RoleIcon className="mr-1 h-3 w-3" />
                      {user.roleName}
                    </Badge>
                  </td>

                  {/* Status */}
                  <td className="px-4 py-3">
                    {user.isBlocked ? (
                      <Badge variant="destructive">{t("settings.blocked")}</Badge>
                    ) : (
                      <Badge variant="secondary">{t("settings.active")}</Badge>
                    )}
                  </td>

                  {/* Last login */}
                  <td className="px-4 py-3 text-muted-foreground">
                    {user.lastLoginAt
                      ? new Date(user.lastLoginAt).toLocaleDateString()
                      : t("settings.never")}
                  </td>

                  {/* Actions */}
                  <td className="px-4 py-3">
                    <div className="flex justify-end gap-1">
                      {!isOwner && !isSelf && (
                        <>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              setRoleDialogUser(user);
                              setSelectedRoleId(user.roleId);
                            }}
                            title={t("settings.changeRole")}
                          >
                            <UserCog className="h-3.5 w-3.5" />
                          </Button>

                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleToggleBlock(user)}
                            title={t(
                              user.isBlocked
                                ? "settings.unblock"
                                : "settings.block",
                            )}
                          >
                            {user.isBlocked ? (
                              <Shield className="h-3.5 w-3.5" />
                            ) : (
                              <ShieldOff className="h-3.5 w-3.5" />
                            )}
                          </Button>

                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => setDeleteDialogUser(user)}
                            title={t("settings.deleteUser")}
                          >
                            <Trash2 className="h-3.5 w-3.5 text-destructive" />
                          </Button>
                        </>
                      )}
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Change role dialog */}
      <Dialog
        open={!!roleDialogUser}
        onClose={() => setRoleDialogUser(null)}
      >
        <DialogHeader>
          <DialogTitle>{t("settings.changeRoleTitle")}</DialogTitle>
          <DialogDescription>
            {t("settings.changeRoleDescription", {
              name: roleDialogUser?.fullName || roleDialogUser?.login || "",
            })}
          </DialogDescription>
        </DialogHeader>
        <DialogContent>
          <div className="space-y-2">
            {roles
              .filter((r) => r.code !== "owner")
              .map((role) => (
                <label
                  key={role.id}
                  className="flex items-center gap-3 rounded-md border border-border p-3 cursor-pointer hover:bg-muted transition-colors"
                >
                  <input
                    type="radio"
                    name="role"
                    value={role.id}
                    checked={selectedRoleId === role.id}
                    onChange={() => setSelectedRoleId(role.id)}
                    className="accent-primary"
                  />
                  <div>
                    <p className="text-sm font-medium">{role.name}</p>
                    <p className="text-xs text-muted-foreground">{role.code}</p>
                  </div>
                </label>
              ))}
          </div>
        </DialogContent>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setRoleDialogUser(null)}
            disabled={actionLoading}
          >
            {t("vault.cancel")}
          </Button>
          <Button onClick={handleRoleChange} disabled={actionLoading}>
            {actionLoading && <Spinner size="sm" />}
            {t("settings.save")}
          </Button>
        </DialogFooter>
      </Dialog>

      {/* Create user dialog */}
      <Dialog open={createOpen} onClose={() => setCreateOpen(false)}>
        <DialogHeader>
          <DialogTitle>{t("settings.createUser")}</DialogTitle>
          <DialogDescription>{t("settings.createUserDescription")}</DialogDescription>
        </DialogHeader>
        <DialogContent>
          <div className="space-y-3">
            <div className="space-y-1.5">
              <Label>{t("login.loginLabel")}</Label>
              <Input
                value={newLogin}
                onChange={(e) => setNewLogin(e.target.value)}
                placeholder="john.doe"
                autoFocus
              />
            </div>
            <div className="space-y-1.5">
              <Label>{t("login.passwordLabel")}</Label>
              <Input
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder="••••••••"
              />
            </div>
            <div className="space-y-1.5">
              <Label>Email</Label>
              <Input
                type="email"
                value={newEmail}
                onChange={(e) => setNewEmail(e.target.value)}
                placeholder="john@example.com"
              />
            </div>
            <div className="space-y-1.5">
              <Label>{t("settings.fullName")}</Label>
              <Input
                value={newFullName}
                onChange={(e) => setNewFullName(e.target.value)}
                placeholder="John Doe"
              />
            </div>
            <div className="space-y-1.5">
              <Label>{t("settings.colRole")}</Label>
              <select
                value={newRoleId}
                onChange={(e) => setNewRoleId(e.target.value)}
                className="w-full h-10 rounded-md border border-input bg-background px-3 text-sm"
              >
                <option value="">{t("settings.defaultRole")}</option>
                {roles
                  .filter((r) => r.code !== "owner")
                  .map((role) => (
                    <option key={role.id} value={role.id}>
                      {role.name}
                    </option>
                  ))}
              </select>
            </div>
          </div>
        </DialogContent>
        <DialogFooter>
          <Button variant="outline" onClick={() => setCreateOpen(false)} disabled={actionLoading}>
            {t("vault.cancel")}
          </Button>
          <Button
            onClick={handleCreateUser}
            disabled={actionLoading || !newLogin.trim() || !newPassword.trim()}
          >
            {actionLoading && <Spinner size="sm" />}
            {t("settings.createUser")}
          </Button>
        </DialogFooter>
      </Dialog>

      {/* Delete user dialog */}
      <Dialog
        open={!!deleteDialogUser}
        onClose={() => setDeleteDialogUser(null)}
      >
        <DialogHeader>
          <DialogTitle>{t("settings.deleteUserTitle")}</DialogTitle>
          <DialogDescription>
            {t("settings.deleteUserWarning", {
              name: deleteDialogUser?.fullName || deleteDialogUser?.login || "",
            })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setDeleteDialogUser(null)}
            disabled={actionLoading}
          >
            {t("vault.cancel")}
          </Button>
          <Button
            variant="destructive"
            onClick={handleDeleteUser}
            disabled={actionLoading}
          >
            {actionLoading && <Spinner size="sm" />}
            {t("settings.deleteConfirm")}
          </Button>
        </DialogFooter>
      </Dialog>
    </div>
  );
}
