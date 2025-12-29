//! External API integrations

pub mod ai_defect_detection;
pub mod weather;

pub use ai_defect_detection::AiDefectDetectionClient;
pub use weather::WeatherClient;
