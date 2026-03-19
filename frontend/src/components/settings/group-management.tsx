import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Users,
  Plus,
  Pencil,
  Trash2,
  UserPlus,
  UserMinus,
  ChevronDown,
  ChevronRight,
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
import { groupApi } from "@/api/groups";
import { userManagementApi } from "@/api/admin";
import type { UserGroupListItem, UserGroupView, GroupMember } from "@/types/group";
import type { AdminUserListItem } from "@/types/admin";

export function GroupManagement() {
  const { t } = useTranslation();

  const [groups, setGroups] = useState<UserGroupListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Expanded group (shows members inline)
  const [expandedGroupId, setExpandedGroupId] = useState<string | null>(null);
  const [expandedGroup, setExpandedGroup] = useState<UserGroupView | null>(null);
  const [loadingMembers, setLoadingMembers] = useState(false);

  // Create/Edit group dialog
  const [groupDialogOpen, setGroupDialogOpen] = useState(false);
  const [groupDialogMode, setGroupDialogMode] = useState<"create" | "edit">("create");
  const [editingGroupId, setEditingGroupId] = useState<string | null>(null);
  const [groupName, setGroupName] = useState("");
  const [groupDescription, setGroupDescription] = useState("");

  // Delete group dialog
  const [deletingGroup, setDeletingGroup] = useState<UserGroupListItem | null>(null);

  // Add member dialog
  const [addMemberGroupId, setAddMemberGroupId] = useState<string | null>(null);
  const [allUsers, setAllUsers] = useState<AdminUserListItem[]>([]);
  const [userSearch, setUserSearch] = useState("");

  const [actionLoading, setActionLoading] = useState(false);

  const loadGroups = useCallback(async () => {
    try {
      const data = await groupApi.list();
      setGroups(data);
    } catch {
      setError(t("settings.errorLoadFailed"));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    loadGroups();
  }, [loadGroups]);

  // Load group details when expanding
  const toggleExpand = async (groupId: string) => {
    if (expandedGroupId === groupId) {
      setExpandedGroupId(null);
      setExpandedGroup(null);
      return;
    }
    setExpandedGroupId(groupId);
    setLoadingMembers(true);
    try {
      const data = await groupApi.get(groupId);
      setExpandedGroup(data);
    } catch {
      setExpandedGroup(null);
    } finally {
      setLoadingMembers(false);
    }
  };

  const reloadExpandedGroup = async () => {
    if (!expandedGroupId) return;
    try {
      const data = await groupApi.get(expandedGroupId);
      setExpandedGroup(data);
    } catch {
      // keep current
    }
  };

  // ─── Group CRUD ───

  const openCreateDialog = () => {
    setGroupDialogMode("create");
    setEditingGroupId(null);
    setGroupName("");
    setGroupDescription("");
    setGroupDialogOpen(true);
    setError("");
  };

  const openEditDialog = (group: UserGroupListItem) => {
    setGroupDialogMode("edit");
    setEditingGroupId(group.id);
    setGroupName(group.name);
    setGroupDescription(group.description);
    setGroupDialogOpen(true);
    setError("");
  };

  const handleSaveGroup = async () => {
    if (!groupName.trim()) return;
    setActionLoading(true);
    setError("");
    try {
      if (groupDialogMode === "create") {
        await groupApi.create({
          name: groupName.trim(),
          description: groupDescription.trim(),
        });
      } else if (editingGroupId) {
        await groupApi.update(editingGroupId, {
          name: groupName.trim(),
          description: groupDescription.trim(),
        });
      }
      setGroupDialogOpen(false);
      await loadGroups();
    } catch (err: unknown) {
      const msg =
        err && typeof err === "object" && "message" in err
          ? String((err as { message: string }).message)
          : t("settings.errorActionFailed");
      setError(msg);
    } finally {
      setActionLoading(false);
    }
  };

  const handleDeleteGroup = async () => {
    if (!deletingGroup) return;
    setActionLoading(true);
    try {
      await groupApi.delete(deletingGroup.id);
      setDeletingGroup(null);
      if (expandedGroupId === deletingGroup.id) {
        setExpandedGroupId(null);
        setExpandedGroup(null);
      }
      await loadGroups();
    } catch {
      setError(t("settings.errorActionFailed"));
    } finally {
      setActionLoading(false);
    }
  };

  // ─── Member management ───

  const openAddMember = async (groupId: string) => {
    setAddMemberGroupId(groupId);
    setUserSearch("");
    try {
      const users = await userManagementApi.list();
      setAllUsers(users);
    } catch {
      setAllUsers([]);
    }
  };

  const handleAddMember = async (userId: string) => {
    if (!addMemberGroupId) return;
    setActionLoading(true);
    try {
      await groupApi.addMember(addMemberGroupId, userId);
      await reloadExpandedGroup();
      await loadGroups();
      setAddMemberGroupId(null);
    } catch (err: unknown) {
      const msg =
        err && typeof err === "object" && "message" in err
          ? String((err as { message: string }).message)
          : t("settings.errorActionFailed");
      setError(msg);
    } finally {
      setActionLoading(false);
    }
  };

  const handleRemoveMember = async (groupId: string, userId: string) => {
    setActionLoading(true);
    try {
      await groupApi.removeMember(groupId, userId);
      await reloadExpandedGroup();
      await loadGroups();
    } catch {
      setError(t("settings.errorActionFailed"));
    } finally {
      setActionLoading(false);
    }
  };

  // Filter users for "add member" dialog (exclude current members)
  const availableUsers = allUsers.filter((u) => {
    if (expandedGroup?.members.some((m) => m.userId === u.id)) return false;
    if (!userSearch.trim()) return true;
    const q = userSearch.toLowerCase();
    return (
      u.login.toLowerCase().includes(q) ||
      u.fullName.toLowerCase().includes(q) ||
      (u.email && u.email.toLowerCase().includes(q))
    );
  });

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
          <h2 className="text-lg font-semibold">{t("groups.title")}</h2>
          <p className="text-sm text-muted-foreground">
            {t("groups.description", { count: groups.length })}
          </p>
        </div>
        <Button onClick={openCreateDialog} size="sm">
          <Plus className="h-4 w-4" />
          {t("groups.create")}
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {/* Groups list */}
      <div className="space-y-2">
        {groups.length === 0 && (
          <div className="rounded-lg border border-border p-8 text-center">
            <Users className="mx-auto h-8 w-8 text-muted-foreground/40" />
            <p className="mt-2 text-sm text-muted-foreground">{t("groups.empty")}</p>
          </div>
        )}

        {groups.map((group) => {
          const isExpanded = expandedGroupId === group.id;

          return (
            <div key={group.id} className="rounded-lg border border-border overflow-hidden">
              {/* Group header */}
              <div className="flex items-center gap-3 px-4 py-3 hover:bg-muted/30 transition-colors">
                <button onClick={() => toggleExpand(group.id)} className="shrink-0">
                  {isExpanded ? (
                    <ChevronDown className="h-4 w-4 text-muted-foreground" />
                  ) : (
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  )}
                </button>

                <button
                  onClick={() => toggleExpand(group.id)}
                  className="flex min-w-0 flex-1 items-center gap-3 text-left"
                >
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                    <Users className="h-4 w-4 text-primary" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="font-medium text-foreground truncate">{group.name}</p>
                    {group.description && (
                      <p className="text-xs text-muted-foreground truncate">
                        {group.description}
                      </p>
                    )}
                  </div>
                  <Badge variant="secondary" className="shrink-0">
                    {t("groups.memberCount", { count: group.memberCount })}
                  </Badge>
                </button>

                <div className="flex shrink-0 gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => openEditDialog(group)}
                    title={t("groups.edit")}
                  >
                    <Pencil className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setDeletingGroup(group)}
                    title={t("groups.delete")}
                  >
                    <Trash2 className="h-3.5 w-3.5 text-destructive" />
                  </Button>
                </div>
              </div>

              {/* Expanded members */}
              {isExpanded && (
                <div className="border-t border-border bg-muted/20 px-4 py-3">
                  {loadingMembers ? (
                    <div className="flex justify-center py-4">
                      <Spinner size="sm" />
                    </div>
                  ) : (
                    <>
                      <div className="flex items-center justify-between mb-2">
                        <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                          {t("groups.members")}
                        </p>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => openAddMember(group.id)}
                          className="h-7 text-xs"
                        >
                          <UserPlus className="h-3.5 w-3.5" />
                          {t("groups.addMember")}
                        </Button>
                      </div>

                      {expandedGroup?.members.length === 0 && (
                        <p className="text-sm text-muted-foreground py-2">
                          {t("groups.noMembers")}
                        </p>
                      )}

                      <div className="space-y-1">
                        {expandedGroup?.members.map((member) => (
                          <MemberRow
                            key={member.id}
                            member={member}
                            onRemove={() => handleRemoveMember(group.id, member.userId)}
                            removing={actionLoading}
                          />
                        ))}
                      </div>
                    </>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Create/Edit group dialog */}
      <Dialog open={groupDialogOpen} onClose={() => setGroupDialogOpen(false)}>
        <DialogHeader>
          <DialogTitle>
            {groupDialogMode === "create" ? t("groups.createTitle") : t("groups.editTitle")}
          </DialogTitle>
          <DialogDescription>
            {groupDialogMode === "create"
              ? t("groups.createDescription")
              : t("groups.editDescription")}
          </DialogDescription>
        </DialogHeader>
        <DialogContent>
          <div className="space-y-3">
            <div className="space-y-1.5">
              <Label>{t("groups.nameLabel")}</Label>
              <Input
                value={groupName}
                onChange={(e) => setGroupName(e.target.value)}
                placeholder={t("groups.namePlaceholder")}
                autoFocus
              />
            </div>
            <div className="space-y-1.5">
              <Label>{t("groups.descriptionLabel")}</Label>
              <Input
                value={groupDescription}
                onChange={(e) => setGroupDescription(e.target.value)}
                placeholder={t("groups.descriptionPlaceholder")}
              />
            </div>
          </div>
        </DialogContent>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setGroupDialogOpen(false)}
            disabled={actionLoading}
          >
            {t("vault.cancel")}
          </Button>
          <Button
            onClick={handleSaveGroup}
            disabled={actionLoading || !groupName.trim()}
          >
            {actionLoading && <Spinner size="sm" />}
            {groupDialogMode === "create" ? t("groups.create") : t("groups.save")}
          </Button>
        </DialogFooter>
      </Dialog>

      {/* Delete group dialog */}
      <Dialog open={!!deletingGroup} onClose={() => setDeletingGroup(null)}>
        <DialogHeader>
          <DialogTitle>{t("groups.deleteTitle")}</DialogTitle>
          <DialogDescription>
            {t("groups.deleteWarning", { name: deletingGroup?.name || "" })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setDeletingGroup(null)}
            disabled={actionLoading}
          >
            {t("vault.cancel")}
          </Button>
          <Button
            variant="destructive"
            onClick={handleDeleteGroup}
            disabled={actionLoading}
          >
            {actionLoading && <Spinner size="sm" />}
            {t("groups.delete")}
          </Button>
        </DialogFooter>
      </Dialog>

      {/* Add member dialog */}
      <Dialog open={!!addMemberGroupId} onClose={() => setAddMemberGroupId(null)}>
        <DialogHeader>
          <DialogTitle>{t("groups.addMemberTitle")}</DialogTitle>
          <DialogDescription>{t("groups.addMemberDescription")}</DialogDescription>
        </DialogHeader>
        <DialogContent>
          <div className="space-y-3">
            <Input
              value={userSearch}
              onChange={(e) => setUserSearch(e.target.value)}
              placeholder={t("groups.searchUsers")}
              autoFocus
            />
            <div className="max-h-60 overflow-y-auto space-y-1">
              {availableUsers.length === 0 && (
                <p className="text-sm text-muted-foreground py-2 text-center">
                  {t("groups.noUsersFound")}
                </p>
              )}
              {availableUsers.map((user) => (
                <button
                  key={user.id}
                  onClick={() => handleAddMember(user.id)}
                  disabled={actionLoading}
                  className="flex w-full items-center gap-3 rounded-md px-3 py-2 text-left hover:bg-muted transition-colors disabled:opacity-50"
                >
                  <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-muted">
                    <Users className="h-3.5 w-3.5 text-muted-foreground" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium truncate">
                      {user.fullName || user.login}
                    </p>
                    <p className="text-xs text-muted-foreground truncate">
                      {user.login}
                      {user.email && ` · ${user.email}`}
                    </p>
                  </div>
                  <UserPlus className="h-4 w-4 shrink-0 text-muted-foreground" />
                </button>
              ))}
            </div>
          </div>
        </DialogContent>
        <DialogFooter>
          <Button variant="outline" onClick={() => setAddMemberGroupId(null)}>
            {t("vault.cancel")}
          </Button>
        </DialogFooter>
      </Dialog>
    </div>
  );
}

// ─── Member Row ───

function MemberRow({
  member,
  onRemove,
  removing,
}: {
  member: GroupMember;
  onRemove: () => void;
  removing: boolean;
}) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-3 rounded-md px-2 py-1.5 hover:bg-muted/50 transition-colors">
      <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-muted">
        <Users className="h-3.5 w-3.5 text-muted-foreground" />
      </div>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium truncate">
          {member.fullName || member.login}
        </p>
        <p className="text-xs text-muted-foreground truncate">
          {member.login}
          {member.email && ` · ${member.email}`}
        </p>
      </div>
      <Button
        variant="ghost"
        size="sm"
        onClick={onRemove}
        disabled={removing}
        title={t("groups.removeMember")}
        className="h-7 w-7 p-0"
      >
        <UserMinus className="h-3.5 w-3.5 text-destructive" />
      </Button>
    </div>
  );
}
