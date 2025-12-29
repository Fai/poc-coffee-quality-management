//! User and role models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::Language;

/// A user account on the platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub business_id: Uuid,
    pub email: Option<String>,
    pub name: String,
    pub phone: Option<String>,
    pub line_user_id: Option<String>,
    pub preferred_language: Language,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A role defining permissions within a business
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub is_system_role: bool,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
}

/// A permission granting access to a resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permission {
    pub resource: Resource,
    pub actions: Vec<Action>,
}

/// Resources that can be accessed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    Plot,
    Harvest,
    Processing,
    Grading,
    Cupping,
    Inventory,
    RoastProfile,
    Report,
    Certification,
    User,
    Role,
    Business,
}

/// Actions that can be performed on resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    View,
    Create,
    Edit,
    Delete,
    Export,
}

/// Default roles created for new businesses
pub fn default_roles() -> Vec<(&'static str, Vec<Permission>)> {
    vec![
        (
            "owner",
            vec![Permission {
                resource: Resource::Business,
                actions: vec![
                    Action::View,
                    Action::Create,
                    Action::Edit,
                    Action::Delete,
                    Action::Export,
                ],
            }],
        ),
        (
            "manager",
            vec![
                Permission {
                    resource: Resource::Plot,
                    actions: vec![Action::View, Action::Create, Action::Edit],
                },
                Permission {
                    resource: Resource::Harvest,
                    actions: vec![Action::View, Action::Create, Action::Edit],
                },
                Permission {
                    resource: Resource::Processing,
                    actions: vec![Action::View, Action::Create, Action::Edit],
                },
                Permission {
                    resource: Resource::Grading,
                    actions: vec![Action::View, Action::Create, Action::Edit],
                },
                Permission {
                    resource: Resource::Cupping,
                    actions: vec![Action::View, Action::Create, Action::Edit],
                },
                Permission {
                    resource: Resource::Inventory,
                    actions: vec![Action::View, Action::Create, Action::Edit],
                },
                Permission {
                    resource: Resource::Report,
                    actions: vec![Action::View, Action::Export],
                },
            ],
        ),
        (
            "worker",
            vec![
                Permission {
                    resource: Resource::Harvest,
                    actions: vec![Action::View, Action::Create],
                },
                Permission {
                    resource: Resource::Processing,
                    actions: vec![Action::View, Action::Create],
                },
                Permission {
                    resource: Resource::Inventory,
                    actions: vec![Action::View],
                },
            ],
        ),
    ]
}
