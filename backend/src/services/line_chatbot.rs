//! LINE Chatbot service for quick data entry
//!
//! Supports quick logging of:
//! - Harvest entries via text commands
//! - Processing entries via text commands
//!
//! Command formats:
//! - Harvest: "harvest [plot_name] [weight_kg] [ripe%]" or "เก็บ [plot_name] [weight_kg] [ripe%]"
//! - Processing: "process [lot_code] [method]" or "แปรรูป [lot_code] [method]"

use chrono::Local;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::harvest::{HarvestService, RecordHarvestInput};
use crate::services::processing::{ProcessingService, StartProcessingInput};
use crate::services::notification::{LineMessage, LineMessagingClient};
use shared::ProcessingMethod;

/// LINE Chatbot service
#[derive(Clone)]
pub struct LineChatbotService {
    db: PgPool,
    line_client: Option<LineMessagingClient>,
}


/// LINE Webhook request body
/// See: https://developers.line.biz/en/reference/messaging-api/#webhook-event-objects
#[derive(Debug, Deserialize)]
pub struct LineWebhookRequest {
    /// User ID of the LINE Official Account that received the webhook event
    pub destination: String,
    /// Array of webhook event objects
    pub events: Vec<LineWebhookEvent>,
}

/// LINE Webhook event
/// See: https://developers.line.biz/en/reference/messaging-api/#common-properties
#[derive(Debug, Deserialize)]
pub struct LineWebhookEvent {
    /// Event type
    #[serde(rename = "type")]
    pub event_type: String,
    /// Token for replying to this event (only for events that can be replied to)
    #[serde(rename = "replyToken")]
    pub reply_token: Option<String>,
    /// Source of the event
    pub source: LineEventSource,
    /// Message object (only for message events)
    pub message: Option<LineEventMessage>,
    /// Time of the event in milliseconds
    pub timestamp: i64,
    /// Channel state: "active" or "standby"
    #[serde(default = "default_mode")]
    pub mode: String,
    /// Webhook event ID (for deduplication)
    #[serde(rename = "webhookEventId")]
    pub webhook_event_id: Option<String>,
    /// Delivery context for redelivery handling
    #[serde(rename = "deliveryContext")]
    pub delivery_context: Option<DeliveryContext>,
}

fn default_mode() -> String {
    "active".to_string()
}

/// Delivery context for webhook events
#[derive(Debug, Deserialize)]
pub struct DeliveryContext {
    /// Whether this is a redelivered event
    #[serde(rename = "isRedelivery")]
    pub is_redelivery: bool,
}

/// LINE event source
#[derive(Debug, Deserialize)]
pub struct LineEventSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    /// Group ID (only for group events)
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    /// Room ID (only for room events)
    #[serde(rename = "roomId")]
    pub room_id: Option<String>,
}

/// LINE event message
#[derive(Debug, Deserialize)]
pub struct LineEventMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub id: String,
    pub text: Option<String>,
    /// Quote token for quoting this message in a reply
    #[serde(rename = "quoteToken")]
    pub quote_token: Option<String>,
}

/// Parsed command from user message
#[derive(Debug, Clone)]
pub enum ChatbotCommand {
    /// Record a harvest: plot_name, weight_kg, ripe_percent
    Harvest {
        plot_name: String,
        weight_kg: Decimal,
        ripe_percent: i32,
    },
    /// Start processing: lot_code, method
    Processing {
        lot_code: String,
        method: ProcessingMethod,
    },
    /// Help command
    Help,
    /// Unknown command
    Unknown(String),
}


/// Result of processing a chatbot command
#[derive(Debug, Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub message_th: String,
    pub entity_id: Option<Uuid>,
}

/// LINE reply message request
#[derive(Debug, Serialize)]
struct LineReplyRequest {
    #[serde(rename = "replyToken")]
    reply_token: String,
    messages: Vec<LineMessage>,
}

impl LineChatbotService {
    /// Create a new LineChatbotService instance
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            line_client: LineMessagingClient::from_env(),
        }
    }

    /// Create with explicit LINE client
    pub fn with_line_client(db: PgPool, line_client: LineMessagingClient) -> Self {
        Self {
            db,
            line_client: Some(line_client),
        }
    }

    /// Process a LINE webhook event
    pub async fn process_webhook(&self, request: LineWebhookRequest) -> AppResult<()> {
        tracing::debug!("Processing webhook for destination: {}", request.destination);
        
        for event in request.events {
            // Skip events in standby mode (for multi-bot scenarios)
            if event.mode == "standby" {
                tracing::debug!("Skipping event in standby mode");
                continue;
            }
            
            // Log redelivery for debugging
            if let Some(ref ctx) = event.delivery_context {
                if ctx.is_redelivery {
                    tracing::info!(
                        "Processing redelivered event: {:?}",
                        event.webhook_event_id
                    );
                }
            }
            
            if event.event_type == "message" {
                if let Some(message) = &event.message {
                    if message.message_type == "text" {
                        if let (Some(text), Some(user_id)) = (&message.text, &event.source.user_id) {
                            let result = self.handle_text_message(user_id, text).await;
                            
                            // Reply to user
                            if let Some(reply_token) = &event.reply_token {
                                let reply_text = match &result {
                                    Ok(r) => format!("{}\n{}", r.message, r.message_th),
                                    Err(e) => format!("Error: {}", e),
                                };
                                let _ = self.reply_message(reply_token, &reply_text).await;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }


    /// Handle a text message from LINE
    pub async fn handle_text_message(
        &self,
        line_user_id: &str,
        text: &str,
    ) -> AppResult<CommandResult> {
        // Get user info from LINE connection
        let user_info = self.get_user_from_line_id(line_user_id).await?;
        
        // Parse the command
        let command = self.parse_command(text);
        
        // Execute the command
        match command {
            ChatbotCommand::Harvest { plot_name, weight_kg, ripe_percent } => {
                self.execute_harvest_command(
                    user_info.user_id,
                    user_info.business_id,
                    &user_info.business_code,
                    &plot_name,
                    weight_kg,
                    ripe_percent,
                ).await
            }
            ChatbotCommand::Processing { lot_code, method } => {
                self.execute_processing_command(
                    user_info.user_id,
                    user_info.business_id,
                    &lot_code,
                    method,
                ).await
            }
            ChatbotCommand::Help => {
                Ok(CommandResult {
                    success: true,
                    message: self.get_help_message_en(),
                    message_th: self.get_help_message_th(),
                    entity_id: None,
                })
            }
            ChatbotCommand::Unknown(msg) => {
                Ok(CommandResult {
                    success: false,
                    message: format!("Unknown command: '{}'. Type 'help' for available commands.", msg),
                    message_th: format!("ไม่รู้จักคำสั่ง: '{}' พิมพ์ 'help' เพื่อดูคำสั่งที่ใช้ได้", msg),
                    entity_id: None,
                })
            }
        }
    }


    /// Parse a text message into a command
    pub fn parse_command(&self, text: &str) -> ChatbotCommand {
        let text = text.trim().to_lowercase();
        let parts: Vec<&str> = text.split_whitespace().collect();
        
        if parts.is_empty() {
            return ChatbotCommand::Unknown(text);
        }
        
        match parts[0] {
            // English commands
            "harvest" | "h" => self.parse_harvest_command(&parts[1..]),
            "process" | "p" => self.parse_processing_command(&parts[1..]),
            "help" | "?" => ChatbotCommand::Help,
            // Thai commands
            "เก็บ" | "เก็บเกี่ยว" => self.parse_harvest_command(&parts[1..]),
            "แปรรูป" | "โปรเซส" => self.parse_processing_command(&parts[1..]),
            "ช่วยเหลือ" | "วิธีใช้" => ChatbotCommand::Help,
            _ => ChatbotCommand::Unknown(text),
        }
    }

    /// Parse harvest command arguments
    fn parse_harvest_command(&self, args: &[&str]) -> ChatbotCommand {
        // Format: harvest [plot_name] [weight_kg] [ripe%]
        // Example: harvest plot1 50 85
        if args.len() < 2 {
            return ChatbotCommand::Unknown(
                "harvest command requires: plot_name weight_kg [ripe%]".to_string()
            );
        }
        
        let plot_name = args[0].to_string();
        
        let weight_kg = match Decimal::from_str(args[1]) {
            Ok(w) if w > Decimal::ZERO => w,
            _ => return ChatbotCommand::Unknown(
                format!("Invalid weight: {}", args[1])
            ),
        };
        
        // Default ripe percent to 80 if not provided
        let ripe_percent = if args.len() > 2 {
            match args[2].parse::<i32>() {
                Ok(p) if (0..=100).contains(&p) => p,
                _ => return ChatbotCommand::Unknown(
                    format!("Invalid ripe percent: {}", args[2])
                ),
            }
        } else {
            80 // Default
        };
        
        ChatbotCommand::Harvest {
            plot_name,
            weight_kg,
            ripe_percent,
        }
    }


    /// Parse processing command arguments
    fn parse_processing_command(&self, args: &[&str]) -> ChatbotCommand {
        // Format: process [lot_code] [method]
        // Example: process CQM-2024-DOI-001 washed
        if args.len() < 2 {
            return ChatbotCommand::Unknown(
                "process command requires: lot_code method".to_string()
            );
        }
        
        let lot_code = args[0].to_uppercase();
        
        let method = match args[1].to_lowercase().as_str() {
            "natural" | "ธรรมชาติ" => ProcessingMethod::Natural,
            "washed" | "ล้าง" => ProcessingMethod::Washed,
            "honey" | "ฮันนี่" => ProcessingMethod::Honey { mucilage_percent: 50 },
            "wet-hulled" | "wethulled" | "กะลาเปียก" => ProcessingMethod::WetHulled,
            "anaerobic" | "ไร้อากาศ" => ProcessingMethod::Anaerobic { hours: 72 },
            _ => return ChatbotCommand::Unknown(
                format!("Unknown processing method: {}. Use: natural, washed, honey, wet-hulled, anaerobic", args[1])
            ),
        };
        
        ChatbotCommand::Processing { lot_code, method }
    }

    /// Get user info from LINE user ID
    async fn get_user_from_line_id(&self, line_user_id: &str) -> AppResult<UserInfo> {
        let row = sqlx::query_as::<_, (Uuid, Uuid, String)>(
            r#"
            SELECT lc.user_id, u.business_id, b.code
            FROM line_connections lc
            JOIN users u ON u.id = lc.user_id
            JOIN businesses b ON b.id = u.business_id
            WHERE lc.line_user_id = $1
            "#,
        )
        .bind(line_user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized {
            message: "LINE account not linked to any user".to_string(),
            message_th: "บัญชี LINE ไม่ได้เชื่อมต่อกับผู้ใช้ใดๆ".to_string(),
        })?;
        
        Ok(UserInfo {
            user_id: row.0,
            business_id: row.1,
            business_code: row.2,
        })
    }


    /// Execute harvest command
    async fn execute_harvest_command(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        business_code: &str,
        plot_name: &str,
        weight_kg: Decimal,
        ripe_percent: i32,
    ) -> AppResult<CommandResult> {
        // Find plot by name
        let plot = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, name FROM plots WHERE business_id = $1 AND LOWER(name) LIKE $2 LIMIT 1"
        )
        .bind(business_id)
        .bind(format!("%{}%", plot_name.to_lowercase()))
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Plot '{}'", plot_name)))?;
        
        // Calculate ripeness (assume remaining is split between underripe and overripe)
        let remaining = 100 - ripe_percent;
        let underripe = remaining / 2;
        let overripe = remaining - underripe;
        
        // Create harvest input
        let input = RecordHarvestInput {
            plot_id: plot.0,
            harvest_date: Local::now().date_naive(),
            picker_name: Some("LINE Quick Entry".to_string()),
            cherry_weight_kg: weight_kg,
            underripe_percent: underripe,
            ripe_percent,
            overripe_percent: overripe,
            weather_snapshot: None,
            notes: Some("Recorded via LINE chatbot".to_string()),
            notes_th: Some("บันทึกผ่าน LINE chatbot".to_string()),
            lot_id: None,
            lot_name: None,
        };
        
        // Record harvest
        let harvest_service = HarvestService::new(self.db.clone());
        let harvest = harvest_service.record_harvest(business_id, business_code, input).await?;
        
        Ok(CommandResult {
            success: true,
            message: format!(
                "✅ Harvest recorded!\nPlot: {}\nWeight: {} kg\nRipeness: {}% ripe\nLot: {}",
                plot.1, weight_kg, ripe_percent, harvest.lot_traceability_code
            ),
            message_th: format!(
                "✅ บันทึกการเก็บเกี่ยวแล้ว!\nแปลง: {}\nน้ำหนัก: {} กก.\nความสุก: {}%\nล็อต: {}",
                plot.1, weight_kg, ripe_percent, harvest.lot_traceability_code
            ),
            entity_id: Some(harvest.id),
        })
    }


    /// Execute processing command
    async fn execute_processing_command(
        &self,
        _user_id: Uuid,
        business_id: Uuid,
        lot_code: &str,
        method: ProcessingMethod,
    ) -> AppResult<CommandResult> {
        // Find lot by traceability code
        let lot = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, name FROM lots WHERE business_id = $1 AND UPPER(traceability_code) = $2"
        )
        .bind(business_id)
        .bind(lot_code.to_uppercase())
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Lot '{}'", lot_code)))?;
        
        // Create processing input
        let input = StartProcessingInput {
            lot_id: lot.0,
            method: method.clone(),
            start_date: Local::now().date_naive(),
            responsible_person: "LINE Quick Entry".to_string(),
            notes: Some("Started via LINE chatbot".to_string()),
            notes_th: Some("เริ่มผ่าน LINE chatbot".to_string()),
        };
        
        // Start processing
        let processing_service = ProcessingService::new(self.db.clone());
        let processing = processing_service.start_processing(business_id, input).await?;
        
        let method_name = match method {
            ProcessingMethod::Natural => "Natural / ธรรมชาติ",
            ProcessingMethod::Washed => "Washed / ล้าง",
            ProcessingMethod::Honey { .. } => "Honey / ฮันนี่",
            ProcessingMethod::WetHulled => "Wet-Hulled / กะลาเปียก",
            ProcessingMethod::Anaerobic { .. } => "Anaerobic / ไร้อากาศ",
            ProcessingMethod::Custom(ref s) => s.as_str(),
        };
        
        Ok(CommandResult {
            success: true,
            message: format!(
                "✅ Processing started!\nLot: {}\nMethod: {}\nStarted: {}",
                lot.1, method_name, processing.start_date
            ),
            message_th: format!(
                "✅ เริ่มการแปรรูปแล้ว!\nล็อต: {}\nวิธี: {}\nเริ่ม: {}",
                lot.1, method_name, processing.start_date
            ),
            entity_id: Some(processing.id),
        })
    }


    /// Reply to a LINE message
    async fn reply_message(&self, reply_token: &str, text: &str) -> AppResult<()> {
        let channel_access_token = std::env::var("LINE_CHANNEL_ACCESS_TOKEN")
            .map_err(|_| AppError::Configuration("LINE_CHANNEL_ACCESS_TOKEN not set".to_string()))?;
        
        let request = LineReplyRequest {
            reply_token: reply_token.to_string(),
            messages: vec![LineMessage::Text { text: text.to_string() }],
        };
        
        let http_client = reqwest::Client::new();
        let response = http_client
            .post("https://api.line.me/v2/bot/message/reply")
            .header("Authorization", format!("Bearer {}", channel_access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("LINE reply error: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalService(format!("LINE reply failed: {}", error_text)));
        }
        
        Ok(())
    }

    /// Get help message in English
    fn get_help_message_en(&self) -> String {
        r#"📋 Coffee QM Quick Commands:

🌿 HARVEST
  harvest [plot] [kg] [ripe%]
  Example: harvest plot1 50 85

⚙️ PROCESSING
  process [lot_code] [method]
  Methods: natural, washed, honey, wet-hulled, anaerobic
  Example: process CQM-2024-DOI-001 washed

❓ HELP
  help or ?"#.to_string()
    }

    /// Get help message in Thai
    fn get_help_message_th(&self) -> String {
        r#"📋 คำสั่งด่วน Coffee QM:

🌿 เก็บเกี่ยว
  เก็บ [แปลง] [กก.] [%สุก]
  ตัวอย่าง: เก็บ แปลง1 50 85

⚙️ แปรรูป
  แปรรูป [รหัสล็อต] [วิธี]
  วิธี: ธรรมชาติ, ล้าง, ฮันนี่, กะลาเปียก, ไร้อากาศ
  ตัวอย่าง: แปรรูป CQM-2024-DOI-001 ล้าง

❓ ช่วยเหลือ
  ช่วยเหลือ หรือ วิธีใช้"#.to_string()
    }
}

/// User info from LINE connection
struct UserInfo {
    user_id: Uuid,
    business_id: Uuid,
    business_code: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only parser that doesn't require a database
    struct CommandParser;

    impl CommandParser {
        fn parse_command(&self, text: &str) -> ChatbotCommand {
            let text = text.trim().to_lowercase();
            let parts: Vec<&str> = text.split_whitespace().collect();
            
            if parts.is_empty() {
                return ChatbotCommand::Unknown(text);
            }
            
            match parts[0] {
                // English commands
                "harvest" | "h" => self.parse_harvest_command(&parts[1..]),
                "process" | "p" => self.parse_processing_command(&parts[1..]),
                "help" | "?" => ChatbotCommand::Help,
                // Thai commands
                "เก็บ" | "เก็บเกี่ยว" => self.parse_harvest_command(&parts[1..]),
                "แปรรูป" | "โปรเซส" => self.parse_processing_command(&parts[1..]),
                "ช่วยเหลือ" | "วิธีใช้" => ChatbotCommand::Help,
                _ => ChatbotCommand::Unknown(text),
            }
        }

        fn parse_harvest_command(&self, args: &[&str]) -> ChatbotCommand {
            if args.len() < 2 {
                return ChatbotCommand::Unknown(
                    "harvest command requires: plot_name weight_kg [ripe%]".to_string()
                );
            }
            
            let plot_name = args[0].to_string();
            
            let weight_kg = match Decimal::from_str(args[1]) {
                Ok(w) if w > Decimal::ZERO => w,
                _ => return ChatbotCommand::Unknown(
                    format!("Invalid weight: {}", args[1])
                ),
            };
            
            let ripe_percent = if args.len() > 2 {
                match args[2].parse::<i32>() {
                    Ok(p) if (0..=100).contains(&p) => p,
                    _ => return ChatbotCommand::Unknown(
                        format!("Invalid ripe percent: {}", args[2])
                    ),
                }
            } else {
                80
            };
            
            ChatbotCommand::Harvest {
                plot_name,
                weight_kg,
                ripe_percent,
            }
        }

        fn parse_processing_command(&self, args: &[&str]) -> ChatbotCommand {
            if args.len() < 2 {
                return ChatbotCommand::Unknown(
                    "process command requires: lot_code method".to_string()
                );
            }
            
            let lot_code = args[0].to_uppercase();
            
            let method = match args[1].to_lowercase().as_str() {
                "natural" | "ธรรมชาติ" => ProcessingMethod::Natural,
                "washed" | "ล้าง" => ProcessingMethod::Washed,
                "honey" | "ฮันนี่" => ProcessingMethod::Honey { mucilage_percent: 50 },
                "wet-hulled" | "wethulled" | "กะลาเปียก" => ProcessingMethod::WetHulled,
                "anaerobic" | "ไร้อากาศ" => ProcessingMethod::Anaerobic { hours: 72 },
                _ => return ChatbotCommand::Unknown(
                    format!("Unknown processing method: {}", args[1])
                ),
            };
            
            ChatbotCommand::Processing { lot_code, method }
        }
    }

    #[test]
    fn test_parse_harvest_command_english() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("harvest plot1 50 85");
        match cmd {
            ChatbotCommand::Harvest { plot_name, weight_kg, ripe_percent } => {
                assert_eq!(plot_name, "plot1");
                assert_eq!(weight_kg, Decimal::from(50));
                assert_eq!(ripe_percent, 85);
            }
            _ => panic!("Expected Harvest command"),
        }
    }

    #[test]
    fn test_parse_harvest_command_thai() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("เก็บ แปลง1 30 90");
        match cmd {
            ChatbotCommand::Harvest { plot_name, weight_kg, ripe_percent } => {
                assert_eq!(plot_name, "แปลง1");
                assert_eq!(weight_kg, Decimal::from(30));
                assert_eq!(ripe_percent, 90);
            }
            _ => panic!("Expected Harvest command"),
        }
    }

    #[test]
    fn test_parse_harvest_command_default_ripe() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("harvest myplot 25");
        match cmd {
            ChatbotCommand::Harvest { plot_name, weight_kg, ripe_percent } => {
                assert_eq!(plot_name, "myplot");
                assert_eq!(weight_kg, Decimal::from(25));
                assert_eq!(ripe_percent, 80); // Default
            }
            _ => panic!("Expected Harvest command"),
        }
    }


    #[test]
    fn test_parse_processing_command_english() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("process CQM-2024-DOI-001 washed");
        match cmd {
            ChatbotCommand::Processing { lot_code, method } => {
                assert_eq!(lot_code, "CQM-2024-DOI-001");
                assert!(matches!(method, ProcessingMethod::Washed));
            }
            _ => panic!("Expected Processing command"),
        }
    }

    #[test]
    fn test_parse_processing_command_thai() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("แปรรูป CQM-2024-DOI-001 ธรรมชาติ");
        match cmd {
            ChatbotCommand::Processing { lot_code, method } => {
                assert_eq!(lot_code, "CQM-2024-DOI-001");
                assert!(matches!(method, ProcessingMethod::Natural));
            }
            _ => panic!("Expected Processing command"),
        }
    }

    #[test]
    fn test_parse_processing_methods() {
        let parser = CommandParser;
        
        // Test all methods
        let methods = vec![
            ("natural", ProcessingMethod::Natural),
            ("washed", ProcessingMethod::Washed),
            ("honey", ProcessingMethod::Honey { mucilage_percent: 50 }),
            ("wet-hulled", ProcessingMethod::WetHulled),
            ("anaerobic", ProcessingMethod::Anaerobic { hours: 72 }),
        ];
        
        for (method_str, expected) in methods {
            let cmd = parser.parse_command(&format!("process LOT001 {}", method_str));
            match cmd {
                ChatbotCommand::Processing { method, .. } => {
                    assert_eq!(std::mem::discriminant(&method), std::mem::discriminant(&expected));
                }
                _ => panic!("Expected Processing command for {}", method_str),
            }
        }
    }

    #[test]
    fn test_parse_help_command() {
        let parser = CommandParser;
        
        assert!(matches!(parser.parse_command("help"), ChatbotCommand::Help));
        assert!(matches!(parser.parse_command("?"), ChatbotCommand::Help));
        assert!(matches!(parser.parse_command("ช่วยเหลือ"), ChatbotCommand::Help));
        assert!(matches!(parser.parse_command("วิธีใช้"), ChatbotCommand::Help));
    }

    #[test]
    fn test_parse_unknown_command() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("unknown command");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_invalid_harvest_weight() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("harvest plot1 invalid 85");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_invalid_ripe_percent() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("harvest plot1 50 150");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_missing_args() {
        let parser = CommandParser;
        
        // Harvest needs at least plot and weight
        let cmd = parser.parse_command("harvest plot1");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
        
        // Process needs lot_code and method
        let cmd = parser.parse_command("process LOT001");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_shorthand_commands() {
        let parser = CommandParser;
        
        // 'h' for harvest
        let cmd = parser.parse_command("h plot1 50 85");
        assert!(matches!(cmd, ChatbotCommand::Harvest { .. }));
        
        // 'p' for process
        let cmd = parser.parse_command("p LOT001 washed");
        assert!(matches!(cmd, ChatbotCommand::Processing { .. }));
    }

    // ========================================================================
    // Additional Edge Case Tests
    // ========================================================================

    #[test]
    fn test_parse_empty_input() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_whitespace_only_input() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("   ");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
        
        let cmd = parser.parse_command("\t\n  ");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_decimal_weight() {
        let parser = CommandParser;
        
        // Decimal weight should work
        let cmd = parser.parse_command("harvest plot1 50.5 85");
        match cmd {
            ChatbotCommand::Harvest { weight_kg, .. } => {
                assert_eq!(weight_kg, Decimal::from_str("50.5").unwrap());
            }
            _ => panic!("Expected Harvest command with decimal weight"),
        }
        
        // Very precise decimal
        let cmd = parser.parse_command("harvest plot1 12.345 80");
        match cmd {
            ChatbotCommand::Harvest { weight_kg, .. } => {
                assert_eq!(weight_kg, Decimal::from_str("12.345").unwrap());
            }
            _ => panic!("Expected Harvest command"),
        }
    }

    #[test]
    fn test_parse_negative_weight() {
        let parser = CommandParser;
        
        // Negative weight should be rejected
        let cmd = parser.parse_command("harvest plot1 -50 85");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_zero_weight() {
        let parser = CommandParser;
        
        // Zero weight should be rejected
        let cmd = parser.parse_command("harvest plot1 0 85");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_boundary_ripe_percent() {
        let parser = CommandParser;
        
        // 0% should be valid
        let cmd = parser.parse_command("harvest plot1 50 0");
        match cmd {
            ChatbotCommand::Harvest { ripe_percent, .. } => {
                assert_eq!(ripe_percent, 0);
            }
            _ => panic!("Expected Harvest command with 0% ripe"),
        }
        
        // 100% should be valid
        let cmd = parser.parse_command("harvest plot1 50 100");
        match cmd {
            ChatbotCommand::Harvest { ripe_percent, .. } => {
                assert_eq!(ripe_percent, 100);
            }
            _ => panic!("Expected Harvest command with 100% ripe"),
        }
        
        // -1% should be invalid
        let cmd = parser.parse_command("harvest plot1 50 -1");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
        
        // 101% should be invalid
        let cmd = parser.parse_command("harvest plot1 50 101");
        assert!(matches!(cmd, ChatbotCommand::Unknown(_)));
    }

    #[test]
    fn test_parse_case_insensitivity() {
        let parser = CommandParser;
        
        // Commands should be case-insensitive
        let cmd = parser.parse_command("HARVEST plot1 50 85");
        assert!(matches!(cmd, ChatbotCommand::Harvest { .. }));
        
        let cmd = parser.parse_command("Harvest plot1 50 85");
        assert!(matches!(cmd, ChatbotCommand::Harvest { .. }));
        
        let cmd = parser.parse_command("PROCESS LOT001 WASHED");
        assert!(matches!(cmd, ChatbotCommand::Processing { .. }));
        
        let cmd = parser.parse_command("HELP");
        assert!(matches!(cmd, ChatbotCommand::Help));
    }

    #[test]
    fn test_parse_extra_whitespace() {
        let parser = CommandParser;
        
        // Extra whitespace between arguments should be handled
        let cmd = parser.parse_command("harvest   plot1   50   85");
        match cmd {
            ChatbotCommand::Harvest { plot_name, weight_kg, ripe_percent } => {
                assert_eq!(plot_name, "plot1");
                assert_eq!(weight_kg, Decimal::from(50));
                assert_eq!(ripe_percent, 85);
            }
            _ => panic!("Expected Harvest command"),
        }
        
        // Leading/trailing whitespace
        let cmd = parser.parse_command("  harvest plot1 50 85  ");
        assert!(matches!(cmd, ChatbotCommand::Harvest { .. }));
    }

    #[test]
    fn test_parse_special_characters_in_plot_name() {
        let parser = CommandParser;
        
        // Plot names with hyphens
        let cmd = parser.parse_command("harvest plot-a1 50 85");
        match cmd {
            ChatbotCommand::Harvest { plot_name, .. } => {
                assert_eq!(plot_name, "plot-a1");
            }
            _ => panic!("Expected Harvest command"),
        }
        
        // Plot names with underscores
        let cmd = parser.parse_command("harvest plot_a1 50 85");
        match cmd {
            ChatbotCommand::Harvest { plot_name, .. } => {
                assert_eq!(plot_name, "plot_a1");
            }
            _ => panic!("Expected Harvest command"),
        }
    }

    #[test]
    fn test_parse_lot_code_uppercase_conversion() {
        let parser = CommandParser;
        
        // Lot codes should be converted to uppercase
        let cmd = parser.parse_command("process cqm-2024-doi-001 washed");
        match cmd {
            ChatbotCommand::Processing { lot_code, .. } => {
                assert_eq!(lot_code, "CQM-2024-DOI-001");
            }
            _ => panic!("Expected Processing command"),
        }
    }

    #[test]
    fn test_parse_unknown_processing_method() {
        let parser = CommandParser;
        
        let cmd = parser.parse_command("process LOT001 unknown_method");
        match cmd {
            ChatbotCommand::Unknown(msg) => {
                assert!(msg.contains("Unknown processing method"));
            }
            _ => panic!("Expected Unknown command for invalid method"),
        }
    }

    #[test]
    fn test_parse_thai_processing_methods() {
        let parser = CommandParser;
        
        // Test Thai method names
        let methods = vec![
            ("ธรรมชาติ", ProcessingMethod::Natural),
            ("ล้าง", ProcessingMethod::Washed),
            ("ฮันนี่", ProcessingMethod::Honey { mucilage_percent: 50 }),
            ("กะลาเปียก", ProcessingMethod::WetHulled),
            ("ไร้อากาศ", ProcessingMethod::Anaerobic { hours: 72 }),
        ];
        
        for (method_str, expected) in methods {
            let cmd = parser.parse_command(&format!("แปรรูป LOT001 {}", method_str));
            match cmd {
                ChatbotCommand::Processing { method, .. } => {
                    assert_eq!(std::mem::discriminant(&method), std::mem::discriminant(&expected));
                }
                _ => panic!("Expected Processing command for Thai method: {}", method_str),
            }
        }
    }

    #[test]
    fn test_parse_alternative_thai_commands() {
        let parser = CommandParser;
        
        // Alternative Thai harvest command
        let cmd = parser.parse_command("เก็บเกี่ยว แปลง1 50 85");
        assert!(matches!(cmd, ChatbotCommand::Harvest { .. }));
        
        // Alternative Thai process command
        let cmd = parser.parse_command("โปรเซส LOT001 washed");
        assert!(matches!(cmd, ChatbotCommand::Processing { .. }));
    }

    #[test]
    fn test_webhook_request_deserialization() {
        let json = r#"{
            "destination": "U1234567890abcdef",
            "events": [
                {
                    "type": "message",
                    "replyToken": "reply-token-123",
                    "source": {
                        "type": "user",
                        "userId": "U9876543210fedcba"
                    },
                    "message": {
                        "type": "text",
                        "id": "msg-123",
                        "text": "harvest plot1 50 85",
                        "quoteToken": "quote-token-456"
                    },
                    "timestamp": 1234567890123,
                    "mode": "active",
                    "webhookEventId": "event-id-789",
                    "deliveryContext": {
                        "isRedelivery": false
                    }
                }
            ]
        }"#;
        
        let request: LineWebhookRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.destination, "U1234567890abcdef");
        assert_eq!(request.events.len(), 1);
        
        let event = &request.events[0];
        assert_eq!(event.event_type, "message");
        assert_eq!(event.reply_token, Some("reply-token-123".to_string()));
        assert_eq!(event.mode, "active");
        assert_eq!(event.webhook_event_id, Some("event-id-789".to_string()));
        assert_eq!(event.delivery_context.as_ref().unwrap().is_redelivery, false);
        
        let source = &event.source;
        assert_eq!(source.source_type, "user");
        assert_eq!(source.user_id, Some("U9876543210fedcba".to_string()));
        
        let message = event.message.as_ref().unwrap();
        assert_eq!(message.message_type, "text");
        assert_eq!(message.id, "msg-123");
        assert_eq!(message.text, Some("harvest plot1 50 85".to_string()));
        assert_eq!(message.quote_token, Some("quote-token-456".to_string()));
    }

    #[test]
    fn test_webhook_request_minimal_fields() {
        // Test with minimal required fields (mode defaults to "active")
        let json = r#"{
            "destination": "U1234567890abcdef",
            "events": [
                {
                    "type": "message",
                    "source": {
                        "type": "user"
                    },
                    "timestamp": 1234567890123
                }
            ]
        }"#;
        
        let request: LineWebhookRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.destination, "U1234567890abcdef");
        assert_eq!(request.events.len(), 1);
        
        let event = &request.events[0];
        assert_eq!(event.mode, "active"); // Default value
        assert!(event.reply_token.is_none());
        assert!(event.message.is_none());
        assert!(event.webhook_event_id.is_none());
        assert!(event.delivery_context.is_none());
    }

    #[test]
    fn test_webhook_group_source() {
        let json = r#"{
            "destination": "U1234567890abcdef",
            "events": [
                {
                    "type": "message",
                    "source": {
                        "type": "group",
                        "groupId": "G1234567890",
                        "userId": "U9876543210"
                    },
                    "timestamp": 1234567890123
                }
            ]
        }"#;
        
        let request: LineWebhookRequest = serde_json::from_str(json).unwrap();
        let source = &request.events[0].source;
        
        assert_eq!(source.source_type, "group");
        assert_eq!(source.group_id, Some("G1234567890".to_string()));
        assert_eq!(source.user_id, Some("U9876543210".to_string()));
    }

    #[test]
    fn test_webhook_redelivery_context() {
        let json = r#"{
            "destination": "U1234567890abcdef",
            "events": [
                {
                    "type": "message",
                    "source": {
                        "type": "user",
                        "userId": "U9876543210"
                    },
                    "timestamp": 1234567890123,
                    "webhookEventId": "event-123",
                    "deliveryContext": {
                        "isRedelivery": true
                    }
                }
            ]
        }"#;
        
        let request: LineWebhookRequest = serde_json::from_str(json).unwrap();
        let event = &request.events[0];
        
        assert!(event.delivery_context.as_ref().unwrap().is_redelivery);
    }
}
