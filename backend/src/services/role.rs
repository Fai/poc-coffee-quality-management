//! Role management service for custom roles and permissions

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Role service for managing custom roles
#[derive(Clone)]
pub struct RoleService {
    db: PgPool,
}

/// Role information
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub name_th: Option<String>,
    pub description: Option<String>,
    pub description_th: Option<String>,
    pub is_system_role: bool,
}

/// Permission information
#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
    pub description_th: Option<String>,
}

/// Input for creating a custom role
#[derive(Debug, Deserialize)]
pub struct CreateRoleInput {
    pub name: String,
    pub name_th: Option<String>,
    pub description: Option<String>,
    pub description_th: Option<String>,
    pub permission_ids: Vec<Uuid>,
}

/// Input for updating a role
#[derive(Debug, Deserialize)]
pub struct UpdateRoleInput {
    pub name: Option<String>,
    pub name_th: Option<String>,
    pub description: Option<String>,
    pub description_th: Option<String>,
    pub permission_ids: Option<Vec<Uuid>>,
}

/// Role with its permissions
#[derive(Debug, Serialize)]
pub struct RoleWithPermissions {
    #[serde(flatten)]
    pub role: Role,
    pub permissions: Vec<Permission>,
}

impl RoleService {
    /// Create a new RoleService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get all roles for a business
    pub async fn get_roles(&self, business_id: Uuid) -> AppResult<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT id, business_id, name, name_th, description, description_th, is_system_role
            FROM roles
            WHERE business_id = $1
            ORDER BY is_system_role DESC, name ASC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(roles)
    }

    /// Get a role by ID with its permissions
    pub async fn get_role_with_permissions(
        &self,
        business_id: Uuid,
        role_id: Uuid,
    ) -> AppResult<RoleWithPermissions> {
        // Get role
        let role = sqlx::query_as::<_, Role>(
            r#"
            SELECT id, business_id, name, name_th, description, description_th, is_system_role
            FROM roles
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(role_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Role".to_string()))?;

        // Get permissions
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT p.id, p.resource, p.action, p.description, p.description_th
            FROM permissions p
            JOIN role_permissions rp ON rp.permission_id = p.id
            WHERE rp.role_id = $1
            ORDER BY p.resource, p.action
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.db)
        .await?;

        Ok(RoleWithPermissions { role, permissions })
    }

    /// Get all available permissions
    pub async fn get_all_permissions(&self) -> AppResult<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT id, resource, action, description, description_th
            FROM permissions
            ORDER BY resource, action
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(permissions)
    }

    /// Create a custom role
    pub async fn create_role(
        &self,
        business_id: Uuid,
        input: CreateRoleInput,
    ) -> AppResult<RoleWithPermissions> {
        // Validate role name doesn't conflict with system roles
        let system_names = ["owner", "manager", "worker"];
        if system_names.contains(&input.name.to_lowercase().as_str()) {
            return Err(AppError::Validation {
                field: "name".to_string(),
                message: "Cannot use reserved role name".to_string(),
                message_th: "ไม่สามารถใช้ชื่อบทบาทที่สงวนไว้".to_string(),
            });
        }

        // Check if role name already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roles WHERE business_id = $1 AND LOWER(name) = LOWER($2)",
        )
        .bind(business_id)
        .bind(&input.name)
        .fetch_one(&self.db)
        .await?;

        if existing > 0 {
            return Err(AppError::Conflict {
                resource: "role".to_string(),
                message: "Role with this name already exists".to_string(),
                message_th: "มีบทบาทชื่อนี้อยู่แล้ว".to_string(),
            });
        }

        // Validate permission IDs exist
        if !input.permission_ids.is_empty() {
            let valid_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM permissions WHERE id = ANY($1)",
            )
            .bind(&input.permission_ids)
            .fetch_one(&self.db)
            .await?;

            if valid_count != input.permission_ids.len() as i64 {
                return Err(AppError::Validation {
                    field: "permission_ids".to_string(),
                    message: "One or more permission IDs are invalid".to_string(),
                    message_th: "รหัสสิทธิ์อย่างน้อยหนึ่งรายการไม่ถูกต้อง".to_string(),
                });
            }
        }

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Create role
        let role_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO roles (business_id, name, name_th, description, description_th, is_system_role)
            VALUES ($1, $2, $3, $4, $5, false)
            RETURNING id
            "#,
        )
        .bind(business_id)
        .bind(&input.name)
        .bind(&input.name_th)
        .bind(&input.description)
        .bind(&input.description_th)
        .fetch_one(&mut *tx)
        .await?;

        // Assign permissions
        for permission_id in &input.permission_ids {
            sqlx::query(
                "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)",
            )
            .bind(role_id)
            .bind(permission_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Return the created role with permissions
        self.get_role_with_permissions(business_id, role_id).await
    }

    /// Update a role (only non-system roles can be fully updated)
    pub async fn update_role(
        &self,
        business_id: Uuid,
        role_id: Uuid,
        input: UpdateRoleInput,
    ) -> AppResult<RoleWithPermissions> {
        // Get existing role
        let existing = sqlx::query_as::<_, Role>(
            "SELECT id, business_id, name, name_th, description, description_th, is_system_role FROM roles WHERE id = $1 AND business_id = $2",
        )
        .bind(role_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Role".to_string()))?;

        // System roles can only have permissions updated, not name/description
        if existing.is_system_role && (input.name.is_some() || input.name_th.is_some()) {
            return Err(AppError::Validation {
                field: "name".to_string(),
                message: "Cannot rename system roles".to_string(),
                message_th: "ไม่สามารถเปลี่ยนชื่อบทบาทระบบได้".to_string(),
            });
        }

        // Validate new name if provided
        if let Some(ref name) = input.name {
            let system_names = ["owner", "manager", "worker"];
            if system_names.contains(&name.to_lowercase().as_str()) {
                return Err(AppError::Validation {
                    field: "name".to_string(),
                    message: "Cannot use reserved role name".to_string(),
                    message_th: "ไม่สามารถใช้ชื่อบทบาทที่สงวนไว้".to_string(),
                });
            }

            // Check for duplicate name
            let duplicate = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM roles WHERE business_id = $1 AND LOWER(name) = LOWER($2) AND id != $3",
            )
            .bind(business_id)
            .bind(name)
            .bind(role_id)
            .fetch_one(&self.db)
            .await?;

            if duplicate > 0 {
                return Err(AppError::Conflict {
                    resource: "role".to_string(),
                    message: "Role with this name already exists".to_string(),
                    message_th: "มีบทบาทชื่อนี้อยู่แล้ว".to_string(),
                });
            }
        }

        // Validate permission IDs if provided
        if let Some(ref permission_ids) = input.permission_ids {
            if !permission_ids.is_empty() {
                let valid_count = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM permissions WHERE id = ANY($1)",
                )
                .bind(permission_ids)
                .fetch_one(&self.db)
                .await?;

                if valid_count != permission_ids.len() as i64 {
                    return Err(AppError::Validation {
                        field: "permission_ids".to_string(),
                        message: "One or more permission IDs are invalid".to_string(),
                        message_th: "รหัสสิทธิ์อย่างน้อยหนึ่งรายการไม่ถูกต้อง".to_string(),
                    });
                }
            }
        }

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Update role fields if not a system role
        if !existing.is_system_role {
            let name = input.name.unwrap_or(existing.name);
            let name_th = input.name_th.or(existing.name_th);
            let description = input.description.or(existing.description);
            let description_th = input.description_th.or(existing.description_th);

            sqlx::query(
                r#"
                UPDATE roles
                SET name = $1, name_th = $2, description = $3, description_th = $4
                WHERE id = $5
                "#,
            )
            .bind(&name)
            .bind(&name_th)
            .bind(&description)
            .bind(&description_th)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
        }

        // Update permissions if provided
        if let Some(permission_ids) = input.permission_ids {
            // Remove existing permissions
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                .bind(role_id)
                .execute(&mut *tx)
                .await?;

            // Add new permissions
            for permission_id in &permission_ids {
                sqlx::query(
                    "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)",
                )
                .bind(role_id)
                .bind(permission_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        // Return updated role with permissions
        self.get_role_with_permissions(business_id, role_id).await
    }

    /// Delete a custom role (system roles cannot be deleted)
    pub async fn delete_role(&self, business_id: Uuid, role_id: Uuid) -> AppResult<()> {
        // Check if role exists and is not a system role
        let role = sqlx::query_as::<_, Role>(
            "SELECT id, business_id, name, name_th, description, description_th, is_system_role FROM roles WHERE id = $1 AND business_id = $2",
        )
        .bind(role_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Role".to_string()))?;

        if role.is_system_role {
            return Err(AppError::Validation {
                field: "role_id".to_string(),
                message: "Cannot delete system roles".to_string(),
                message_th: "ไม่สามารถลบบทบาทระบบได้".to_string(),
            });
        }

        // Check if any users are assigned to this role
        let user_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role_id = $1",
        )
        .bind(role_id)
        .fetch_one(&self.db)
        .await?;

        if user_count > 0 {
            return Err(AppError::Validation {
                field: "role_id".to_string(),
                message: format!("Cannot delete role: {} users are assigned to it", user_count),
                message_th: format!("ไม่สามารถลบบทบาท: มีผู้ใช้ {} คนที่ใช้บทบาทนี้", user_count),
            });
        }

        // Delete role (cascade will delete role_permissions)
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(role_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
