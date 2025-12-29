//! AI Defect Detection Client
//!
//! Client for the AWS-hosted AI defect detection microservice.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use shared::{AiDefectDetection, DefectBreakdown, GradeClassification};

use crate::error::{AppError, AppResult};

/// Client for AI defect detection microservice
#[derive(Clone)]
pub struct AiDefectDetectionClient {
    api_endpoint: String,
    api_key: String,
    http_client: Client,
}

/// Request to detect defects in an image
#[derive(Debug, Serialize)]
pub struct DetectDefectsRequest {
    pub image_base64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_weight_grams: Option<f64>,
}

/// Response from defect detection API
#[derive(Debug, Deserialize)]
pub struct DetectDefectsResponse {
    pub request_id: String,
    pub detection: AiDetectionResult,
    pub suggested_grade: String,
}

/// AI detection result from the API
#[derive(Debug, Deserialize)]
pub struct AiDetectionResult {
    pub request_id: String,
    pub image_url: String,
    pub detected_beans: i32,
    pub defect_breakdown: DefectBreakdownResponse,
    pub category1_count: i32,
    pub category2_count: i32,
    pub confidence_score: f32,
    pub processing_time_ms: i32,
    pub annotated_image_url: Option<String>,
}

/// Defect breakdown from API response
#[derive(Debug, Deserialize)]
pub struct DefectBreakdownResponse {
    // Category 1 (Primary) Defects
    pub full_black: i32,
    pub full_sour: i32,
    pub pod_cherry: i32,
    pub large_stones: i32,
    pub medium_stones: i32,
    pub large_sticks: i32,
    pub medium_sticks: i32,
    // Category 2 (Secondary) Defects
    pub partial_black: i32,
    pub partial_sour: i32,
    pub parchment: i32,
    pub floater: i32,
    pub immature: i32,
    pub withered: i32,
    pub shell: i32,
    pub broken: i32,
    pub chipped: i32,
    pub cut: i32,
    pub insect_damage: i32,
    pub husk: i32,
}

impl From<DefectBreakdownResponse> for DefectBreakdown {
    fn from(r: DefectBreakdownResponse) -> Self {
        DefectBreakdown {
            full_black: r.full_black,
            full_sour: r.full_sour,
            pod_cherry: r.pod_cherry,
            large_stones: r.large_stones,
            medium_stones: r.medium_stones,
            large_sticks: r.large_sticks,
            medium_sticks: r.medium_sticks,
            partial_black: r.partial_black,
            partial_sour: r.partial_sour,
            parchment: r.parchment,
            floater: r.floater,
            immature: r.immature,
            withered: r.withered,
            shell: r.shell,
            broken: r.broken,
            chipped: r.chipped,
            cut: r.cut,
            insect_damage: r.insect_damage,
            husk: r.husk,
        }
    }
}

impl From<AiDetectionResult> for AiDefectDetection {
    fn from(r: AiDetectionResult) -> Self {
        AiDefectDetection {
            request_id: r.request_id,
            image_url: r.image_url,
            detected_beans: r.detected_beans,
            defect_breakdown: r.defect_breakdown.into(),
            category1_count: r.category1_count,
            category2_count: r.category2_count,
            confidence_score: r.confidence_score,
            processing_time_ms: r.processing_time_ms,
            annotated_image_url: r.annotated_image_url,
        }
    }
}

/// Detection status for async processing
#[derive(Debug, Deserialize)]
pub struct DetectionStatus {
    pub request_id: String,
    pub status: String,
    pub detection: Option<AiDetectionResult>,
    pub error: Option<String>,
}

impl AiDefectDetectionClient {
    /// Create a new AI defect detection client
    pub fn new(api_endpoint: String, api_key: String) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_endpoint,
            api_key,
            http_client,
        }
    }

    /// Create a client from environment variables
    pub fn from_env() -> Option<Self> {
        let api_endpoint = std::env::var("CQM__AI_DETECTION__API_ENDPOINT").ok()?;
        let api_key = std::env::var("CQM__AI_DETECTION__API_KEY").ok()?;

        Some(Self::new(api_endpoint, api_key))
    }

    /// Send image for AI defect detection
    pub async fn detect_defects(
        &self,
        request: DetectDefectsRequest,
    ) -> AppResult<DetectDefectsResponse> {
        let response = self
            .http_client
            .post(&self.api_endpoint)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::AiDetectionError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::AiDetectionError(format!(
                "API returned {}: {}",
                status, body
            )));
        }

        let result: DetectDefectsResponse = response
            .json()
            .await
            .map_err(|e| AppError::AiDetectionError(format!("Failed to parse response: {}", e)))?;

        Ok(result)
    }

    /// Get detection status for async processing
    pub async fn get_detection_status(&self, request_id: &str) -> AppResult<DetectionStatus> {
        let url = format!("{}/status/{}", self.api_endpoint, request_id);

        let response = self
            .http_client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| AppError::AiDetectionError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::AiDetectionError(format!(
                "API returned {}: {}",
                status, body
            )));
        }

        let result: DetectionStatus = response
            .json()
            .await
            .map_err(|e| AppError::AiDetectionError(format!("Failed to parse response: {}", e)))?;

        Ok(result)
    }

    /// Convert suggested grade string to GradeClassification
    pub fn parse_grade(grade_str: &str) -> GradeClassification {
        match grade_str {
            "specialty_grade" => GradeClassification::SpecialtyGrade,
            "premium_grade" => GradeClassification::PremiumGrade,
            "exchange_grade" => GradeClassification::ExchangeGrade,
            "below_standard" => GradeClassification::BelowStandard,
            _ => GradeClassification::OffGrade,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_parsing() {
        assert_eq!(
            AiDefectDetectionClient::parse_grade("specialty_grade"),
            GradeClassification::SpecialtyGrade
        );
        assert_eq!(
            AiDefectDetectionClient::parse_grade("premium_grade"),
            GradeClassification::PremiumGrade
        );
        assert_eq!(
            AiDefectDetectionClient::parse_grade("exchange_grade"),
            GradeClassification::ExchangeGrade
        );
        assert_eq!(
            AiDefectDetectionClient::parse_grade("below_standard"),
            GradeClassification::BelowStandard
        );
        assert_eq!(
            AiDefectDetectionClient::parse_grade("off_grade"),
            GradeClassification::OffGrade
        );
        assert_eq!(
            AiDefectDetectionClient::parse_grade("unknown"),
            GradeClassification::OffGrade
        );
    }

    #[test]
    fn test_defect_breakdown_conversion() {
        let response = DefectBreakdownResponse {
            full_black: 1,
            full_sour: 2,
            pod_cherry: 0,
            large_stones: 0,
            medium_stones: 0,
            large_sticks: 0,
            medium_sticks: 0,
            partial_black: 3,
            partial_sour: 1,
            parchment: 0,
            floater: 0,
            immature: 2,
            withered: 1,
            shell: 0,
            broken: 4,
            chipped: 2,
            cut: 0,
            insect_damage: 1,
            husk: 0,
        };

        let breakdown: DefectBreakdown = response.into();
        assert_eq!(breakdown.full_black, 1);
        assert_eq!(breakdown.full_sour, 2);
        assert_eq!(breakdown.broken, 4);
        assert_eq!(breakdown.insect_damage, 1);
    }
}
