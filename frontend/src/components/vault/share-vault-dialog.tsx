import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { UserPlus, Trash2, Shield, Crown, Users, UsersRound } from "lucide-react";
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
import { sharingApi } from "@/api/sharing";
import { groupApi } from "@/api/groups";
import { userManagementApi } from "@/api/admin";
import { api } from "@/api/client";
import { useAuthStore } from "@/stores/auth";
import type { VaultMember, ResourceAccessLevel } from "@/types/sharing";
import type { AdminUserListItem } from "@/types/admin";
import type { UserGroupListItem, AccessGrantView } from "@/types/group";
import { cn } from "@/lib/utils";

interface Props {
  open: boolean;
  onClose: () => void;
  vaultId: string;
  vaultName: string;
  creatorId: string;
}

const ACCESS_BADGE_VARIANT: Record<string, "default" | "secondary" | "outline" | "destructive"> = {
  admin: "default",
  manage: "secondary",
  write: "outline",
  read: "outline",
  forbidden: "destructive",
};

export function ShareVaultDialog({ open, onClose, vaultId, vaultName, creatorId }: Props) {
  const { t } = useTranslation();
  const currentUserId = useAuthStore((s) => s.user?.id);

  const [members, setMembers] = useState<VaultMember[]>([]);
  const [groupGrants, setGroupGrants] = useState<AccessGrantView[]>([]);
  const [accessLevels, setAccessLevels] = useState<ResourceAccessLevel[]>([]);
  const [allUsers, setAllUsers] = useState<AdminUserListItem[]>([]);
  const [allGroups, setAllGroups] = useState<UserGroupListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Add form
  const [addMode, setAddMode] = useState<"user" | "group">("user");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedUserId, setSelectedUserId] = useState("");
  const [selectedGroupId, setSelectedGroupId] = useState("");
  const [selectedAccessId, setSelectedAccessId] = useState("");
  const [adding, setAdding] = useState(false);

  const loadData = useCallback(async () => {
    try {
      const [membersData, levelsData, usersData, groupsData] = await Promise.all([
        sharingApi.listMembers(vaultId),
        sharingApi.listAccessLevels(),
        userManagementApi.list().catch(() => [] as AdminUserListItem[]),
        groupApi.list().catch(() => [] as UserGroupListItem[]),
      ]);

      // Load group grants from the unified grants table
      let groupGrantsData: AccessGrantView[] = [];
      try {
        const allGrants = await api.get<AccessGrantView[]>(`/sharing/${vaultId}/grants`);
        groupGrantsData = allGrants.filter((g) => g.granteeType === "group");
      } catch {
        // Endpoint may not exist yet; groups grants loaded separately
      }

      setMembers(membersData);
      setGroupGrants(groupGrantsData);
      setAccessLevels(levelsData);
      setAllUsers(usersData);
      setAllGroups(groupsData);

      // Default access level to "read"
      const readLevel = levelsData.find((l) => l.code === "read");
      if (readLevel && !selectedAccessId) {
        setSelectedAccessId(readLevel.id);
      }
    } catch {
      setError(t("sharing.errorLoadFailed"));
    } finally {
      setLoading(false);
    }
  }, [vaultId, t]);

  useEffect(() => {
    if (open) {
      setLoading(true);
      setError("");
      loadData();
    }
  }, [open, loadData]);

  // Filter users available to add (not already members, not creator)
  const memberUserIds = new Set(members.map((m) => m.userId));
  const availableUsers = allUsers.filter(
    (u) =>
      u.id !== creatorId &&
      !memberUserIds.has(u.id) &&
      (searchQuery === "" ||
        u.login.toLowerCase().includes(searchQuery.toLowerCase()) ||
        u.fullName.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (u.email && u.email.toLowerCase().includes(searchQuery.toLowerCase()))),
  );

  // Filter groups available to add
  const grantedGroupIds = new Set(groupGrants.map((g) => g.groupId));
  const availableGroups = allGroups.filter(
    (g) =>
      !grantedGroupIds.has(g.id) &&
      (searchQuery === "" ||
        g.name.toLowerCase().includes(searchQuery.toLowerCase())),
  );

  const handleAddMember = async () => {
    if (addMode === "user" && (!selectedUserId || !selectedAccessId)) return;
    if (addMode === "group" && (!selectedGroupId || !selectedAccessId)) return;

    setAdding(true);
    setError("");
    try {
      if (addMode === "user") {
        await sharingApi.share(vaultId, selectedUserId, selectedAccessId);
        setSelectedUserId("");
      } else {
        await api.post(`/sharing/${vaultId}/groups`, {
          groupId: selectedGroupId,
          resourceAccessId: selectedAccessId,
        });
        setSelectedGroupId("");
      }
      setSearchQuery("");
      await loadData();
    } catch {
      setError(t("sharing.errorShareFailed"));
    } finally {
      setAdding(false);
    }
  };

  const handleChangeAccess = async (userId: string, accessId: string) => {
    try {
      await sharingApi.updateAccess(vaultId, userId, accessId);
      await loadData();
    } catch {
      setError(t("sharing.errorUpdateFailed"));
    }
  };

  const handleRevoke = async (userId: string) => {
    try {
      await sharingApi.revokeAccess(vaultId, userId);
      await loadData();
    } catch {
      setError(t("sharing.errorRevokeFailed"));
    }
  };

  const handleRevokeGroup = async (grantId: string) => {
    try {
      await api.delete(`/sharing/grants/${grantId}`);
      await loadData();
    } catch {
      setError(t("sharing.errorRevokeFailed"));
    }
  };

  const isOwner = currentUserId === creatorId;

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogHeader>
        <DialogTitle>{t("sharing.title")}</DialogTitle>
        <DialogDescription>
          {t("sharing.description", { name: vaultName })}
        </DialogDescription>
      </DialogHeader>

      <DialogContent>
        {loading ? (
          <div className="flex justify-center py-8">
            <Spinner size="md" />
          </div>
        ) : (
          <div className="space-y-6">
            {/* Owner row */}
            <div className="flex items-center justify-between rounded-md border border-border p-3 bg-muted/30">
              <div className="flex items-center gap-2">
                <Crown className="h-4 w-4 text-amber-500" />
                <span className="text-sm font-medium">
                  {allUsers.find((u) => u.id === creatorId)?.login || t("sharing.owner")}
                </span>
              </div>
              <Badge variant="default">{t("sharing.owner")}</Badge>
            </div>

            {/* Current user members */}
            {members.length > 0 && (
              <div className="space-y-2">
                <h3 className="text-sm font-medium text-muted-foreground">
                  <Users className="inline mr-1 h-3.5 w-3.5" />
                  {t("sharing.members", { count: members.length })}
                </h3>
                {members.map((member) => (
                  <div
                    key={member.id}
                    className="flex items-center justify-between rounded-md border border-border p-3"
                  >
                    <div>
                      <p className="text-sm font-medium">
                        {member.fullName || member.login}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {member.login}
                        {member.email && ` · ${member.email}`}
                      </p>
                    </div>
                    <div className="flex items-center gap-2">
                      {isOwner ? (
                        <>
                          <select
                            value={member.resourceAccessId}
                            onChange={(e) =>
                              handleChangeAccess(member.userId, e.target.value)
                            }
                            className="h-8 rounded-md border border-input bg-background px-2 text-xs"
                          >
                            {accessLevels.map((level) => (
                              <option key={level.id} value={level.id}>
                                {level.name}
                              </option>
                            ))}
                          </select>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleRevoke(member.userId)}
                            title={t("sharing.revoke")}
                          >
                            <Trash2 className="h-3.5 w-3.5 text-destructive" />
                          </Button>
                        </>
                      ) : (
                        <Badge
                          variant={
                            ACCESS_BADGE_VARIANT[member.accessCode] || "outline"
                          }
                        >
                          <Shield className="mr-1 h-3 w-3" />
                          {member.accessName}
                        </Badge>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* Current group grants */}
            {groupGrants.length > 0 && (
              <div className="space-y-2">
                <h3 className="text-sm font-medium text-muted-foreground">
                  <UsersRound className="inline mr-1 h-3.5 w-3.5" />
                  {t("sharing.groupMembers", { count: groupGrants.length })}
                </h3>
                {groupGrants.map((grant) => (
                  <div
                    key={grant.id}
                    className="flex items-center justify-between rounded-md border border-border p-3"
                  >
                    <div className="flex items-center gap-2">
                      <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                        <UsersRound className="h-3.5 w-3.5 text-primary" />
                      </div>
                      <div>
                        <p className="text-sm font-medium">{grant.granteeName}</p>
                        <p className="text-xs text-muted-foreground">{t("sharing.group")}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {isOwner ? (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleRevokeGroup(grant.id)}
                          title={t("sharing.revoke")}
                        >
                          <Trash2 className="h-3.5 w-3.5 text-destructive" />
                        </Button>
                      ) : null}
                      <Badge
                        variant={ACCESS_BADGE_VARIANT[grant.accessCode] || "outline"}
                      >
                        <Shield className="mr-1 h-3 w-3" />
                        {grant.accessName}
                      </Badge>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* Add user/group (only for owners) */}
            {isOwner && (
              <div className="space-y-3 border-t border-border pt-4">
                <h3 className="text-sm font-medium">
                  <UserPlus className="inline mr-1 h-4 w-4" />
                  {t("sharing.addMember")}
                </h3>

                {/* User/Group toggle */}
                <div className="flex gap-1 rounded-lg border border-border p-0.5">
                  <button
                    onClick={() => {
                      setAddMode("user");
                      setSearchQuery("");
                      setSelectedGroupId("");
                    }}
                    className={cn(
                      "flex-1 flex items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
                      addMode === "user"
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:text-foreground",
                    )}
                  >
                    <Users className="h-3.5 w-3.5" />
                    {t("sharing.addUser")}
                  </button>
                  <button
                    onClick={() => {
                      setAddMode("group");
                      setSearchQuery("");
                      setSelectedUserId("");
                    }}
                    className={cn(
                      "flex-1 flex items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors",
                      addMode === "group"
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:text-foreground",
                    )}
                  >
                    <UsersRound className="h-3.5 w-3.5" />
                    {t("sharing.addGroup")}
                  </button>
                </div>

                {addMode === "user" ? (
                  <div className="space-y-2">
                    <Label>{t("sharing.searchUser")}</Label>
                    <Input
                      value={searchQuery}
                      onChange={(e) => {
                        setSearchQuery(e.target.value);
                        setSelectedUserId("");
                      }}
                      placeholder={t("sharing.searchPlaceholder")}
                      maxLength={100}
                    />

                    {searchQuery.length >= 2 && availableUsers.length > 0 && (
                      <div className="max-h-32 overflow-y-auto rounded-md border border-border">
                        {availableUsers.slice(0, 10).map((user) => (
                          <button
                            key={user.id}
                            onClick={() => {
                              setSelectedUserId(user.id);
                              setSearchQuery(user.fullName || user.login);
                            }}
                            className={`w-full text-left px-3 py-2 text-sm hover:bg-muted transition-colors ${
                              selectedUserId === user.id ? "bg-muted" : ""
                            }`}
                          >
                            <span className="font-medium">
                              {user.fullName || user.login}
                            </span>
                            <span className="ml-2 text-muted-foreground">
                              {user.login}
                            </span>
                          </button>
                        ))}
                      </div>
                    )}
                    {searchQuery.length >= 2 && availableUsers.length === 0 && (
                      <p className="text-xs text-muted-foreground">
                        {t("sharing.noUsersFound")}
                      </p>
                    )}
                  </div>
                ) : (
                  <div className="space-y-2">
                    <Label>{t("sharing.selectGroup")}</Label>
                    {availableGroups.length === 0 ? (
                      <p className="text-xs text-muted-foreground py-2">
                        {t("sharing.noGroupsAvailable")}
                      </p>
                    ) : (
                      <div className="max-h-32 overflow-y-auto rounded-md border border-border">
                        {availableGroups.map((group) => (
                          <button
                            key={group.id}
                            onClick={() => setSelectedGroupId(group.id)}
                            className={cn(
                              "flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-muted transition-colors text-left",
                              selectedGroupId === group.id && "bg-muted",
                            )}
                          >
                            <UsersRound className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                            <div className="min-w-0 flex-1">
                              <span className="font-medium">{group.name}</span>
                              <span className="ml-2 text-muted-foreground text-xs">
                                {t("groups.memberCount", { count: group.memberCount })}
                              </span>
                            </div>
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                )}

                <div className="flex items-end gap-2">
                  <div className="flex-1 space-y-2">
                    <Label>{t("sharing.accessLevel")}</Label>
                    <select
                      value={selectedAccessId}
                      onChange={(e) => setSelectedAccessId(e.target.value)}
                      className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                    >
                      {accessLevels.map((level) => (
                        <option key={level.id} value={level.id}>
                          {level.name}
                        </option>
                      ))}
                    </select>
                  </div>
                  <Button
                    onClick={handleAddMember}
                    disabled={
                      (addMode === "user" && !selectedUserId) ||
                      (addMode === "group" && !selectedGroupId) ||
                      !selectedAccessId ||
                      adding
                    }
                  >
                    {adding ? (
                      <Spinner size="sm" />
                    ) : (
                      <UserPlus className="h-4 w-4" />
                    )}
                    {t("sharing.add")}
                  </Button>
                </div>
              </div>
            )}

            {error && <p className="text-sm text-destructive">{error}</p>}
          </div>
        )}
      </DialogContent>

      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t("sharing.close")}
        </Button>
      </DialogFooter>
    </Dialog>
  );
}
