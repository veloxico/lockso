use serde::{Deserialize, Serialize};

/// Authentication method used to create a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "PascalCase")]
pub enum AuthMethod {
    Local,
    Sso,
    Ldap,
}

/// Client type for session tracking.
///
/// Different client types have different session policies:
/// - Web: inactivity TTL enforced
/// - Desktop/Mobile/Extension/API: no inactivity timeout
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "PascalCase")]
pub enum ClientType {
    Web,
    Desktop,
    BrowserExtension,
    MobileApplication,
    Api,
}

impl ClientType {
    /// Parse a client type string, returning None for invalid values.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Web" => Some(Self::Web),
            "Desktop" => Some(Self::Desktop),
            "BrowserExtension" => Some(Self::BrowserExtension),
            "MobileApplication" => Some(Self::MobileApplication),
            "Api" => Some(Self::Api),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "Web",
            Self::Desktop => "Desktop",
            Self::BrowserExtension => "BrowserExtension",
            Self::MobileApplication => "MobileApplication",
            Self::Api => "Api",
        }
    }
}

/// User signup method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "PascalCase")]
pub enum UserSignupType {
    Default,
    Sso,
    Ldap,
}

/// Default user roles created at bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultUserRole {
    Owner,
    Admin,
    User,
}

impl DefaultUserRole {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::User => "user",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Admin => "Administrator",
            Self::User => "User",
        }
    }
}

/// Default resource access levels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultPermissionAccess {
    Admin,
    Manage,
    Write,
    Read,
    Forbidden,
}

impl DefaultPermissionAccess {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Manage => "manage",
            Self::Write => "write",
            Self::Read => "read",
            Self::Forbidden => "forbidden",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Admin => "Admin",
            Self::Manage => "Manage",
            Self::Write => "Write",
            Self::Read => "Read Only",
            Self::Forbidden => "Forbidden",
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            Self::Admin => 100,
            Self::Manage => 80,
            Self::Write => 60,
            Self::Read => 40,
            Self::Forbidden => 0,
        }
    }
}

/// Default vault types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultVaultType {
    Organization,
    Personal,
    PrivateShared,
}

impl DefaultVaultType {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Organization => "organization",
            Self::Personal => "personal",
            Self::PrivateShared => "private_shared",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Organization => "Organization",
            Self::Personal => "Personal",
            Self::PrivateShared => "Private Shared",
        }
    }
}

/// Session attribute types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionAttribute {
    DesktopApplication,
    TransitionSession,
    BrowserExtensionAuth,
    MobileApplicationAuth,
    DesktopApplicationAuth,
    SsoCheckAuth,
}

/// Last authentication type with interval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LastAuthenticationType {
    /// Main authentication — 5 minute recheck interval.
    Main,
    /// Mobile authentication — 30 second recheck interval.
    Mobile,
}
