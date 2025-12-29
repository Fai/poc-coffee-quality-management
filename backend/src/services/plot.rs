//! Plot management service for farm and plot operations

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Plot service for managing farm plots
#[derive(Clone)]
pub struct PlotService {
    db: PgPool,
}

/// Plot information
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Plot {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub area_rai: Option<Decimal>,
    pub altitude_meters: Option<i32>,
    pub shade_coverage_percent: Option<i32>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Plot variety information
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlotVariety {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub variety: String,
    pub variety_th: Option<String>,
    pub planting_date: Option<NaiveDate>,
    pub tree_count: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Plot with its varieties
#[derive(Debug, Clone, Serialize)]
pub struct PlotWithVarieties {
    #[serde(flatten)]
    pub plot: Plot,
    pub varieties: Vec<PlotVariety>,
}

/// Input for creating a plot
#[derive(Debug, Deserialize)]
pub struct CreatePlotInput {
    pub name: String,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub area_rai: Option<Decimal>,
    pub altitude_meters: Option<i32>,
    pub shade_coverage_percent: Option<i32>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub varieties: Option<Vec<CreateVarietyInput>>,
}

/// Input for creating a variety
#[derive(Debug, Deserialize)]
pub struct CreateVarietyInput {
    pub variety: String,
    pub variety_th: Option<String>,
    pub planting_date: Option<NaiveDate>,
    pub tree_count: Option<i32>,
    pub notes: Option<String>,
}

/// Input for updating a plot
#[derive(Debug, Deserialize)]
pub struct UpdatePlotInput {
    pub name: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub area_rai: Option<Decimal>,
    pub altitude_meters: Option<i32>,
    pub shade_coverage_percent: Option<i32>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Plot statistics
#[derive(Debug, Serialize)]
pub struct PlotStatistics {
    pub plot_id: Uuid,
    pub total_harvests: i64,
    pub total_cherry_weight_kg: Decimal,
    pub average_yield_per_rai: Option<Decimal>,
    pub last_harvest_date: Option<NaiveDate>,
    pub harvest_history: Vec<HarvestSummary>,
}

/// Harvest summary for statistics
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HarvestSummary {
    pub harvest_date: NaiveDate,
    pub cherry_weight_kg: Decimal,
    pub ripe_percent: i32,
}

impl PlotService {
    /// Create a new PlotService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get all plots for a business
    pub async fn get_plots(&self, business_id: Uuid) -> AppResult<Vec<Plot>> {
        let plots = sqlx::query_as::<_, Plot>(
            r#"
            SELECT id, business_id, name, latitude, longitude, area_rai, 
                   altitude_meters, shade_coverage_percent, notes, notes_th,
                   created_at, updated_at
            FROM plots
            WHERE business_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(plots)
    }

    /// Get a plot by ID with its varieties
    pub async fn get_plot_with_varieties(
        &self,
        business_id: Uuid,
        plot_id: Uuid,
    ) -> AppResult<PlotWithVarieties> {
        // Get plot
        let plot = sqlx::query_as::<_, Plot>(
            r#"
            SELECT id, business_id, name, latitude, longitude, area_rai,
                   altitude_meters, shade_coverage_percent, notes, notes_th,
                   created_at, updated_at
            FROM plots
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(plot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Plot".to_string()))?;

        // Get varieties
        let varieties = sqlx::query_as::<_, PlotVariety>(
            r#"
            SELECT id, plot_id, variety, variety_th, planting_date, tree_count, notes, created_at
            FROM plot_varieties
            WHERE plot_id = $1
            ORDER BY variety ASC
            "#,
        )
        .bind(plot_id)
        .fetch_all(&self.db)
        .await?;

        Ok(PlotWithVarieties { plot, varieties })
    }

    /// Create a new plot
    pub async fn create_plot(
        &self,
        business_id: Uuid,
        input: CreatePlotInput,
    ) -> AppResult<PlotWithVarieties> {
        // Validate input
        if input.name.trim().is_empty() {
            return Err(AppError::Validation {
                field: "name".to_string(),
                message: "Plot name cannot be empty".to_string(),
                message_th: "ชื่อแปลงไม่สามารถว่างได้".to_string(),
            });
        }

        // Validate shade coverage
        if let Some(shade) = input.shade_coverage_percent {
            if shade < 0 || shade > 100 {
                return Err(AppError::Validation {
                    field: "shade_coverage_percent".to_string(),
                    message: "Shade coverage must be between 0 and 100".to_string(),
                    message_th: "เปอร์เซ็นต์ร่มเงาต้องอยู่ระหว่าง 0 ถึง 100".to_string(),
                });
            }
        }

        // Validate altitude for Thai coffee regions
        if let Some(altitude) = input.altitude_meters {
            if altitude < 0 || altitude > 3000 {
                return Err(AppError::Validation {
                    field: "altitude_meters".to_string(),
                    message: "Altitude must be between 0 and 3000 meters".to_string(),
                    message_th: "ความสูงต้องอยู่ระหว่าง 0 ถึง 3000 เมตร".to_string(),
                });
            }
        }

        // Check for duplicate name
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM plots WHERE business_id = $1 AND LOWER(name) = LOWER($2)",
        )
        .bind(business_id)
        .bind(&input.name)
        .fetch_one(&self.db)
        .await?;

        if existing > 0 {
            return Err(AppError::Conflict {
                resource: "plot".to_string(),
                message: "A plot with this name already exists".to_string(),
                message_th: "มีแปลงชื่อนี้อยู่แล้ว".to_string(),
            });
        }

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Create plot
        let plot_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO plots (business_id, name, latitude, longitude, area_rai,
                              altitude_meters, shade_coverage_percent, notes, notes_th)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(business_id)
        .bind(&input.name)
        .bind(&input.latitude)
        .bind(&input.longitude)
        .bind(&input.area_rai)
        .bind(&input.altitude_meters)
        .bind(&input.shade_coverage_percent)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&mut *tx)
        .await?;

        // Add varieties if provided
        if let Some(varieties) = input.varieties {
            for variety_input in varieties {
                sqlx::query(
                    r#"
                    INSERT INTO plot_varieties (plot_id, variety, variety_th, planting_date, tree_count, notes)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                )
                .bind(plot_id)
                .bind(&variety_input.variety)
                .bind(&variety_input.variety_th)
                .bind(&variety_input.planting_date)
                .bind(&variety_input.tree_count)
                .bind(&variety_input.notes)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        // Return the created plot with varieties
        self.get_plot_with_varieties(business_id, plot_id).await
    }

    /// Update a plot
    pub async fn update_plot(
        &self,
        business_id: Uuid,
        plot_id: Uuid,
        input: UpdatePlotInput,
    ) -> AppResult<PlotWithVarieties> {
        // Check if plot exists
        let existing = sqlx::query_as::<_, Plot>(
            "SELECT id, business_id, name, latitude, longitude, area_rai, altitude_meters, shade_coverage_percent, notes, notes_th, created_at, updated_at FROM plots WHERE id = $1 AND business_id = $2",
        )
        .bind(plot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Plot".to_string()))?;

        // Validate new name if provided
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(AppError::Validation {
                    field: "name".to_string(),
                    message: "Plot name cannot be empty".to_string(),
                    message_th: "ชื่อแปลงไม่สามารถว่างได้".to_string(),
                });
            }

            // Check for duplicate name
            let duplicate = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM plots WHERE business_id = $1 AND LOWER(name) = LOWER($2) AND id != $3",
            )
            .bind(business_id)
            .bind(name)
            .bind(plot_id)
            .fetch_one(&self.db)
            .await?;

            if duplicate > 0 {
                return Err(AppError::Conflict {
                    resource: "plot".to_string(),
                    message: "A plot with this name already exists".to_string(),
                    message_th: "มีแปลงชื่อนี้อยู่แล้ว".to_string(),
                });
            }
        }

        // Validate shade coverage
        if let Some(shade) = input.shade_coverage_percent {
            if shade < 0 || shade > 100 {
                return Err(AppError::Validation {
                    field: "shade_coverage_percent".to_string(),
                    message: "Shade coverage must be between 0 and 100".to_string(),
                    message_th: "เปอร์เซ็นต์ร่มเงาต้องอยู่ระหว่าง 0 ถึง 100".to_string(),
                });
            }
        }

        // Update plot
        let name = input.name.unwrap_or(existing.name);
        let latitude = input.latitude.or(existing.latitude);
        let longitude = input.longitude.or(existing.longitude);
        let area_rai = input.area_rai.or(existing.area_rai);
        let altitude_meters = input.altitude_meters.or(existing.altitude_meters);
        let shade_coverage_percent = input.shade_coverage_percent.or(existing.shade_coverage_percent);
        let notes = input.notes.or(existing.notes);
        let notes_th = input.notes_th.or(existing.notes_th);

        sqlx::query(
            r#"
            UPDATE plots
            SET name = $1, latitude = $2, longitude = $3, area_rai = $4,
                altitude_meters = $5, shade_coverage_percent = $6, notes = $7, notes_th = $8
            WHERE id = $9
            "#,
        )
        .bind(&name)
        .bind(&latitude)
        .bind(&longitude)
        .bind(&area_rai)
        .bind(&altitude_meters)
        .bind(&shade_coverage_percent)
        .bind(&notes)
        .bind(&notes_th)
        .bind(plot_id)
        .execute(&self.db)
        .await?;

        // Return updated plot with varieties
        self.get_plot_with_varieties(business_id, plot_id).await
    }

    /// Delete a plot
    pub async fn delete_plot(&self, business_id: Uuid, plot_id: Uuid) -> AppResult<()> {
        // Check if plot exists
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM plots WHERE id = $1 AND business_id = $2",
        )
        .bind(plot_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if exists == 0 {
            return Err(AppError::NotFound("Plot".to_string()));
        }

        // Check if plot has harvests
        let harvest_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM harvests WHERE plot_id = $1",
        )
        .bind(plot_id)
        .fetch_one(&self.db)
        .await?;

        if harvest_count > 0 {
            return Err(AppError::Validation {
                field: "plot_id".to_string(),
                message: format!("Cannot delete plot: {} harvests are linked to it", harvest_count),
                message_th: format!("ไม่สามารถลบแปลง: มีการเก็บเกี่ยว {} รายการที่เชื่อมโยงอยู่", harvest_count),
            });
        }

        // Delete plot (cascade will delete varieties)
        sqlx::query("DELETE FROM plots WHERE id = $1")
            .bind(plot_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Add a variety to a plot
    pub async fn add_variety(
        &self,
        business_id: Uuid,
        plot_id: Uuid,
        input: CreateVarietyInput,
    ) -> AppResult<PlotVariety> {
        // Check if plot exists and belongs to business
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM plots WHERE id = $1 AND business_id = $2",
        )
        .bind(plot_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if exists == 0 {
            return Err(AppError::NotFound("Plot".to_string()));
        }

        // Check for duplicate variety
        let duplicate = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM plot_varieties WHERE plot_id = $1 AND LOWER(variety) = LOWER($2)",
        )
        .bind(plot_id)
        .bind(&input.variety)
        .fetch_one(&self.db)
        .await?;

        if duplicate > 0 {
            return Err(AppError::Conflict {
                resource: "variety".to_string(),
                message: "This variety already exists for this plot".to_string(),
                message_th: "พันธุ์นี้มีอยู่แล้วในแปลงนี้".to_string(),
            });
        }

        // Insert variety
        let variety = sqlx::query_as::<_, PlotVariety>(
            r#"
            INSERT INTO plot_varieties (plot_id, variety, variety_th, planting_date, tree_count, notes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, plot_id, variety, variety_th, planting_date, tree_count, notes, created_at
            "#,
        )
        .bind(plot_id)
        .bind(&input.variety)
        .bind(&input.variety_th)
        .bind(&input.planting_date)
        .bind(&input.tree_count)
        .bind(&input.notes)
        .fetch_one(&self.db)
        .await?;

        Ok(variety)
    }

    /// Remove a variety from a plot
    pub async fn remove_variety(
        &self,
        business_id: Uuid,
        plot_id: Uuid,
        variety_id: Uuid,
    ) -> AppResult<()> {
        // Check if plot exists and belongs to business
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM plots WHERE id = $1 AND business_id = $2",
        )
        .bind(plot_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if exists == 0 {
            return Err(AppError::NotFound("Plot".to_string()));
        }

        // Delete variety
        let result = sqlx::query("DELETE FROM plot_varieties WHERE id = $1 AND plot_id = $2")
            .bind(variety_id)
            .bind(plot_id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Variety".to_string()));
        }

        Ok(())
    }

    /// Get plot statistics including harvest history
    pub async fn get_plot_statistics(
        &self,
        business_id: Uuid,
        plot_id: Uuid,
    ) -> AppResult<PlotStatistics> {
        // Check if plot exists
        let plot = sqlx::query_as::<_, Plot>(
            "SELECT id, business_id, name, latitude, longitude, area_rai, altitude_meters, shade_coverage_percent, notes, notes_th, created_at, updated_at FROM plots WHERE id = $1 AND business_id = $2",
        )
        .bind(plot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Plot".to_string()))?;

        // Get harvest statistics
        let stats = sqlx::query_as::<_, (i64, Decimal, Option<NaiveDate>)>(
            r#"
            SELECT 
                COUNT(*) as total_harvests,
                COALESCE(SUM(cherry_weight_kg), 0) as total_cherry_weight_kg,
                MAX(harvest_date) as last_harvest_date
            FROM harvests
            WHERE plot_id = $1
            "#,
        )
        .bind(plot_id)
        .fetch_one(&self.db)
        .await?;

        // Calculate average yield per rai
        let average_yield_per_rai = if let Some(area) = plot.area_rai {
            if area > Decimal::ZERO && stats.0 > 0 {
                Some(stats.1 / area)
            } else {
                None
            }
        } else {
            None
        };

        // Get harvest history (last 10 harvests)
        let harvest_history = sqlx::query_as::<_, HarvestSummary>(
            r#"
            SELECT harvest_date, cherry_weight_kg, ripe_percent
            FROM harvests
            WHERE plot_id = $1
            ORDER BY harvest_date DESC
            LIMIT 10
            "#,
        )
        .bind(plot_id)
        .fetch_all(&self.db)
        .await?;

        Ok(PlotStatistics {
            plot_id,
            total_harvests: stats.0,
            total_cherry_weight_kg: stats.1,
            average_yield_per_rai,
            last_harvest_date: stats.2,
            harvest_history,
        })
    }
}
