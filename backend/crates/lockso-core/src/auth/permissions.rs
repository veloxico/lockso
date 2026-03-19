/// All 83 role permission items.
///
/// These define what actions a user with a given role can perform system-wide.
/// (85 minus 2 license-related ones removed for open-source)
pub const ALL_ROLE_PERMISSIONS: &[&str] = &[
    // User & Account
    "user:own:login:edit",
    "user:own:interfaceSettings:use",
    "apiTokens:use",
    "browserExtension:use",
    "mobileApplication:use",
    "desktopApplication:use",
    "user:resetPassword",
    "user:resetMasterKey",
    "user:resetTwoFactorAuth",
    // Vault Management
    "vault:create",
    "vault:delete",
    "vault:settings",
    "vaultType:create",
    "vaultType:settings",
    "vaultType:manage",
    "vault:access:sharing",
    // Folder Management
    "folder:create",
    "folder:move",
    "folder:delete",
    // Item Management
    "item:create",
    "item:move",
    "item:delete",
    "item:copy",
    "item:passwordValue:read",
    // Sharing Features
    "shortcut:create",
    "shortcut:delete",
    "link:create",
    "link:manage",
    "inbox:send",
    "inbox:manage",
    // User Management
    "user:create",
    "user:delete",
    "user:manage",
    "user:invite",
    "user:block",
    "user:role:any:manage",
    // User Groups
    "userGroup:create",
    "userGroup:delete",
    "userGroup:manage",
    "userGroup:setState",
    "userGroup:user:manage",
    // Roles
    "userRole:read",
    "userRole:manage",
    "user:role:set",
    // LDAP
    "ldapServer:create",
    "ldapServer:delete",
    "ldapServer:manage",
    "ldapUser:manage",
    "ldapUserGroup:manage",
    // Settings & Admin
    "settings:read",
    "settings:manage",
    "settings:email:manage",
    "settings:sso:manage",
    // Activity Log
    "activityLog:read",
    "activityLog:manage",
    // Tags
    "tag:create",
    "tag:delete",
    "tag:manage",
    // Attachments
    "attachment:create",
    "attachment:delete",
    // Snapshots
    "snapshot:manage",
    // SSO
    "sso:manage",
    // WebAuthn
    "webauthn:manage",
    // Notifications
    "notification:manage",
    // Favicon
    "favicon:manage",
    // Import/Export
    "import:manage",
    "export:manage",
    // Trash
    "trash:manage",
    "trash:restore",
    // API Tokens
    "apiToken:create",
    "apiToken:delete",
    "apiToken:manage",
    // Resource Access
    "resourceAccess:create",
    "resourceAccess:delete",
    "resourceAccess:manage",
    // Vault Types management
    "vaultType:delete",
    // Browser Extension
    "browserExtension:manage",
    // Custom fields
    "customField:manage",
    // Offline
    "offlineAccess:use",
];

/// 28 resource-level permission items.
///
/// These define what a user can do within a specific vault/folder.
pub const ALL_RESOURCE_PERMISSIONS: &[&str] = &[
    // Directory
    "directory:read",
    "directory:accessList:read",
    "directory:accessList:edit",
    // Item
    "item:read",
    "item:create",
    "item:edit",
    "item:delete",
    "item:copy",
    "item:move",
    "item:passwordValue:read",
    "item:attachment:read",
    "item:attachment:create",
    "item:attachment:delete",
    "item:snapshot:view",
    // Folder
    "folder:create",
    "folder:move",
    "folder:delete",
    "folder:inbox:send",
    "folder:shortcut:create",
    "folder:shortcut:revoke",
    "folder:accessList:edit",
    // Vault
    "vault:usersAccess:read",
    "vault:usersAccess:edit",
    "vault:userGroupsAccess:edit",
    // Tags (within vault)
    "tag:create",
    "tag:delete",
    "tag:manage",
    // Custom fields (within vault)
    "customField:manage",
];

/// User-level permissions for the "User" role (limited subset).
pub fn user_role_permissions() -> Vec<&'static str> {
    vec![
        "user:own:login:edit",
        "user:own:interfaceSettings:use",
        "browserExtension:use",
        "mobileApplication:use",
        "desktopApplication:use",
        "vault:create",
        "folder:create",
        "folder:move",
        "folder:delete",
        "item:create",
        "item:move",
        "item:delete",
        "item:copy",
        "item:passwordValue:read",
        "shortcut:create",
        "shortcut:delete",
        "inbox:send",
        "tag:create",
        "attachment:create",
        "attachment:delete",
        "offlineAccess:use",
    ]
}
