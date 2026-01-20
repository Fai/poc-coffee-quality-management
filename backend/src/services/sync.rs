//! Offline sync service for PWA support
//! Handles change tracking, delta sync, and conflict resolution

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Sync service for offline support
#[derive(Clone)]
pub struct SyncService {
    db: PgPool,
}

/// A change record from sync_log
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncChange {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: String,
    pub entity_version: i64,
    pub data: Option<serde_json::Value>,
    pub changed_at: DateTime<Utc>,
}

/// Client's pending change to apply
#[derive(Debug, Deserialize)]
pub struct PendingChange {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: String,
    pub client_version: i64,
    pub data: serde_json::Value,
    pub changed_at: DateTime<Utc>,
}

/// Sync conflict requiring resolution
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SyncConflict {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub local_version: serde_json::Value,
    pub local_changed_at: DateTime<Utc>,
    pub server_version: serde_json::Value,
    pub server_changed_at: DateTime<Utc>,
    pub server_entity_version: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// Result of applying sync changes
#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub applied: Vec<Uuid>,
    pub conflicts: Vec<SyncConflict>,
    pub server_version: i64,
}

/// Conflict resolution choice
#[derive(Debug, Deserialize)]
pub enum ConflictResolution {
    KeepLocal,
    KeepServer,
    Merge(serde_json::Value),
}

/// Sync state for a device
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SyncState {
    pub user_id: Uuid,
    pub device_id: String,
    pub last_sync_at: DateTime<Utc>,
    pub last_sync_version: i64,
}

impl SyncService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get changes since a specific version for delta sync
    pub async fn get_changes_since(
        &self,
        business_id: Uuid,
        since_version: i64,
        limit: i32,
    ) -> AppResult<Vec<SyncChange>> {
        let changes = sqlx::query_as::<_, SyncChange>(
            r#"
            SELECT entity_type, entity_id, operation, entity_version, data, changed_at
            FROM sync_log
            WHERE business_id = $1 AND entity_version > $2
            ORDER BY entity_version ASC
            LIMIT $3
            "#,
        )
        .bind(business_id)
        .bind(since_version)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(changes)
    }

    /// Get current sync state for a device
    pub async fn get_sync_state(
        &self,
        user_id: Uuid,
        device_id: &str,
    ) -> AppResult<Option<SyncState>> {
        let state = sqlx::query_as::<_, SyncState>(
            "SELECT user_id, device_id, last_sync_at, last_sync_version FROM sync_state WHERE user_id = $1 AND device_id = $2",
        )
        .bind(user_id)
        .bind(device_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(state)
    }

    /// Update sync state after successful sync
    pub async fn update_sync_state(
        &self,
        user_id: Uuid,
        device_id: &str,
        version: i64,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sync_state (user_id, device_id, last_sync_at, last_sync_version)
            VALUES ($1, $2, NOW(), $3)
            ON CONFLICT (user_id, device_id) 
            DO UPDATE SET last_sync_at = NOW(), last_sync_version = $3
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .bind(version)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Apply pending changes from client, detecting conflicts
    pub async fn apply_changes(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        changes: Vec<PendingChange>,
    ) -> AppResult<SyncResult> {
        let mut applied = Vec::new();
        let mut conflicts = Vec::new();

        for change in changes {
            match self.apply_single_change(user_id, business_id, &change).await {
                Ok(entity_id) => applied.push(entity_id),
                Err(AppError::SyncConflict { conflict }) => conflicts.push(conflict),
                Err(e) => return Err(e),
            }
        }

        // Get current server version
        let server_version: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(entity_version), 0) FROM sync_log WHERE business_id = $1")
            .bind(business_id)
            .fetch_one(&self.db)
            .await?;

        Ok(SyncResult {
            applied,
            conflicts,
            server_version,
        })
    }

    /// Apply a single change, checking for conflicts
    async fn apply_single_change(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        change: &PendingChange,
    ) -> AppResult<Uuid> {
        // Check for conflict
        let conflict_check = sqlx::query_as::<_, (bool, Option<i64>, Option<serde_json::Value>)>(
            "SELECT * FROM check_sync_conflict($1, $2, $3)",
        )
        .bind(&change.entity_type)
        .bind(change.entity_id)
        .bind(change.client_version)
        .fetch_one(&self.db)
        .await?;

        if conflict_check.0 {
            // Conflict detected - create conflict record
            let conflict = self
                .create_conflict(
                    user_id,
                    business_id,
                    change,
                    conflict_check.1.unwrap_or(0),
                    conflict_check.2.unwrap_or(serde_json::Value::Null),
                )
                .await?;

            return Err(AppError::SyncConflict { conflict });
        }

        // No conflict - apply the change
        self.execute_change(change).await?;
        Ok(change.entity_id)
    }

    /// Create a conflict record for user resolution
    async fn create_conflict(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        change: &PendingChange,
        server_version: i64,
        server_data: serde_json::Value,
    ) -> AppResult<SyncConflict> {
        let conflict = sqlx::query_as::<_, SyncConflict>(
            r#"
            INSERT INTO sync_conflicts (
                business_id, user_id, entity_type, entity_id,
                local_version, local_changed_at, server_version, server_changed_at, server_entity_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8)
            RETURNING id, entity_type, entity_id, local_version, local_changed_at, 
                      server_version, server_changed_at, server_entity_version, status, created_at
            "#,
        )
        .bind(business_id)
        .bind(user_id)
        .bind(&change.entity_type)
        .bind(change.entity_id)
        .bind(&change.data)
        .bind(change.changed_at)
        .bind(&server_data)
        .bind(server_version)
        .fetch_one(&self.db)
        .await?;

        Ok(conflict)
    }

    /// Execute a change (create/update/delete)
    async fn execute_change(&self, change: &PendingChange) -> AppResult<()> {
        match change.operation.as_str() {
            "create" => self.execute_create(change).await,
            "update" => self.execute_update(change).await,
            "delete" => self.execute_delete(change).await,
            _ => Err(AppError::Validation {
                field: "operation".to_string(),
                message: format!("Invalid operation: {}", change.operation),
            }),
        }
    }

    async fn execute_create(&self, change: &PendingChange) -> AppResult<()> {
        // Dynamic insert based on entity type
        let table = Self::validate_table_name(&change.entity_type)?;
        let columns: Vec<String> = change.data.as_object()
            .ok_or_else(|| AppError::Validation {
                field: "data".to_string(),
                message: "Data must be an object".to_string(),
            })?
            .keys()
            .cloned()
            .collect();

        let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT (id) DO NOTHING",
            table,
            columns.join(", "),
            placeholders.join(", ")
        );

        let mut q = sqlx::query(&query);
        for col in &columns {
            if let Some(val) = change.data.get(col) {
                q = q.bind(val.clone());
            }
        }
        q.execute(&self.db).await?;
        Ok(())
    }

    async fn execute_update(&self, change: &PendingChange) -> AppResult<()> {
        let table = Self::validate_table_name(&change.entity_type)?;
        let obj = change.data.as_object().ok_or_else(|| AppError::Validation {
            field: "data".to_string(),
            message: "Data must be an object".to_string(),
        })?;

        let updates: Vec<String> = obj.keys()
            .filter(|k| *k != "id")
            .enumerate()
            .map(|(i, k)| format!("{} = ${}", k, i + 1))
            .collect();

        if updates.is_empty() {
            return Ok(());
        }

        let query = format!(
            "UPDATE {} SET {} WHERE id = ${}",
            table,
            updates.join(", "),
            updates.len() + 1
        );

        let mut q = sqlx::query(&query);
        for (key, val) in obj.iter().filter(|(k, _)| *k != "id") {
            q = q.bind(val.clone());
        }
        q = q.bind(change.entity_id);
        q.execute(&self.db).await?;
        Ok(())
    }

    async fn execute_delete(&self, change: &PendingChange) -> AppResult<()> {
        let table = Self::validate_table_name(&change.entity_type)?;
        let query = format!("DELETE FROM {} WHERE id = $1", table);
        sqlx::query(&query)
            .bind(change.entity_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    fn validate_table_name(entity_type: &str) -> AppResult<&str> {
        match entity_type {
            "plots" | "lots" | "harvests" | "processing_records" | "green_bean_grades"
            | "cupping_sessions" | "cupping_samples" | "inventory_transactions" | "roast_sessions" => {
                Ok(entity_type)
            }
            _ => Err(AppError::Validation {
                field: "entity_type".to_string(),
                message: format!("Invalid entity type: {}", entity_type),
            }),
        }
    }

    /// Get pending conflicts for a user
    pub async fn get_pending_conflicts(&self, user_id: Uuid) -> AppResult<Vec<SyncConflict>> {
        let conflicts = sqlx::query_as::<_, SyncConflict>(
            r#"
            SELECT id, entity_type, entity_id, local_version, local_changed_at,
                   server_version, server_changed_at, server_entity_version, status, created_at
            FROM sync_conflicts
            WHERE user_id = $1 AND status = 'pending'
            ORDER BY created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(conflicts)
    }

    /// Resolve a sync conflict
    pub async fn resolve_conflict(
        &self,
        conflict_id: Uuid,
        user_id: Uuid,
        resolution: ConflictResolution,
    ) -> AppResult<()> {
        let conflict = sqlx::query_as::<_, SyncConflict>(
            "SELECT * FROM sync_conflicts WHERE id = $1 AND user_id = $2 AND status = 'pending'",
        )
        .bind(conflict_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::NotFound {
            resource: "conflict".to_string(),
            id: conflict_id.to_string(),
        })?;

        let (status, resolved_data) = match resolution {
            ConflictResolution::KeepLocal => {
                // Apply local version
                let change = PendingChange {
                    entity_type: conflict.entity_type.clone(),
                    entity_id: conflict.entity_id,
                    operation: "update".to_string(),
                    client_version: conflict.server_entity_version,
                    data: conflict.local_version.clone(),
                    changed_at: Utc::now(),
                };
                self.execute_change(&change).await?;
                ("resolved_local", conflict.local_version)
            }
            ConflictResolution::KeepServer => {
                // Server version already applied, just mark resolved
                ("resolved_server", conflict.server_version)
            }
            ConflictResolution::Merge(merged_data) => {
                // Apply merged version
                let change = PendingChange {
                    entity_type: conflict.entity_type.clone(),
                    entity_id: conflict.entity_id,
                    operation: "update".to_string(),
                    client_version: conflict.server_entity_version,
                    data: merged_data.clone(),
                    changed_at: Utc::now(),
                };
                self.execute_change(&change).await?;
                ("resolved_merged", merged_data)
            }
        };

        sqlx::query(
            "UPDATE sync_conflicts SET status = $1, resolved_at = NOW(), resolved_data = $2 WHERE id = $3",
        )
        .bind(status)
        .bind(&resolved_data)
        .bind(conflict_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
