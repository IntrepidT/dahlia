use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;
use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Teacher,
    Guest,
    User,
    SuperAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountStatus {
    Pending,
    Active,
    Suspended,
    Deleted,
}

impl FromStr for UserRole {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "teacher" => Ok(UserRole::Teacher),
            "guest" => Ok(UserRole::Guest),
            "user" => Ok(UserRole::User),
            "superadmin" => Ok(UserRole::SuperAdmin),
            _ => Err(()),
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role_str = match self {
            UserRole::Admin => "admin",
            UserRole::Teacher => "teacher",
            UserRole::Guest => "guest",
            UserRole::User => "user",
            UserRole::SuperAdmin => "superadmin",
        };
        write!(f, "{}", role_str)
    }
}

impl AccountStatus {
    pub fn to_string(&self) -> String {
        match self {
            AccountStatus::Pending => "pending".to_string(),
            AccountStatus::Active => "active".to_string(),
            AccountStatus::Suspended => "suspended".to_string(),
            AccountStatus::Deleted => "deleted".to_string(),
        }
    }

    pub fn from_str(status: &str) -> Self {
        match status.to_lowercase().as_str() {
            "pending" => AccountStatus::Pending,
            "active" => AccountStatus::Active,
            "suspended" => AccountStatus::Suspended,
            "deleted" => AccountStatus::Deleted,
            _ => AccountStatus::Pending,
        }
    }
}

// Full User struct - contains ALL user data including sensitive information
// Used for: database operations, admin functions, complete user management
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing, default)]
    pub password_hash: String,
    pub role: UserRole,
    pub password_salt: Option<String>,
    pub account_status: AccountStatus,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub phone_verified: bool,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl User {
    pub fn new(
        id: i64,
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
    ) -> Self {
        User {
            id,
            username,
            email,
            password_hash,
            role,
            password_salt: None,
            account_status: AccountStatus::Active,
            email_verified: false,
            phone_number: None,
            phone_verified: false,
            display_name: None,
            first_name: None,
            last_name: None,
        }
    }

    pub fn is_user(&self) -> bool {
        matches!(
            self.role,
            UserRole::User | UserRole::Teacher | UserRole::Admin | UserRole::SuperAdmin
        )
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::SuperAdmin)
    }

    pub fn is_teacher(&self) -> bool {
        matches!(
            self.role,
            UserRole::Teacher | UserRole::Admin | UserRole::SuperAdmin
        )
    }

    pub fn is_super_admin(&self) -> bool {
        self.role == UserRole::SuperAdmin
    }

    pub fn is_guest(&self) -> bool {
        self.role == UserRole::Guest
    }

    // Convert to session-safe version (without sensitive data)
    pub fn to_session_user(&self) -> SessionUser {
        SessionUser {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            role: self.role,
            display_name: self.display_name.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl SessionUser {
    pub fn is_user(&self) -> bool {
        matches!(
            self.role,
            UserRole::User | UserRole::Teacher | UserRole::Admin | UserRole::SuperAdmin
        )
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::SuperAdmin)
    }

    pub fn is_teacher(&self) -> bool {
        matches!(
            self.role,
            UserRole::Teacher | UserRole::Admin | UserRole::SuperAdmin
        )
    }

    pub fn is_super_admin(&self) -> bool {
        self.role == UserRole::SuperAdmin
    }

    pub fn is_guest(&self) -> bool {
        self.role == UserRole::Guest
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer}, encode::IsNull};
        use sqlx::prelude::*;

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for UserRole {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, sqlx::error::BoxDynError> {
                let role_str = self.to_string();
                Encode::<Postgres>::encode_by_ref(&role_str, buf)
            }
        }

        impl <'r> sqlx::decode::Decode<'r, Postgres> for UserRole {
            fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
                let role_str: String = Decode::<Postgres>::decode(value)?;
                role_str.parse().map_err(|_| {
                    sqlx::error::BoxDynError::from(format!("Invalid UserRole: {}", role_str))
                })
            }
        }

        impl Type<Postgres> for UserRole {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("user_role_enum")
            }
        }
    }
}
