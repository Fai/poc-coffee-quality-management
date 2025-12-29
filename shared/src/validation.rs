//! Validation utilities for Coffee Quality Management Platform
//!
//! Includes Thailand-specific validations for compliance with local regulations.

use rust_decimal::Decimal;

use crate::models::{DefectCount, GradeClassification, RipenessAssessment};

// ============================================================================
// Coffee Quality Validations
// ============================================================================

/// Validate that ripeness percentages sum to 100
pub fn validate_ripeness(ripeness: &RipenessAssessment) -> Result<(), &'static str> {
    let total = ripeness.underripe_percent + ripeness.ripe_percent + ripeness.overripe_percent;
    if total != 100 {
        return Err("Ripeness percentages must sum to 100");
    }
    if ripeness.underripe_percent < 0 || ripeness.ripe_percent < 0 || ripeness.overripe_percent < 0
    {
        return Err("Ripeness percentages cannot be negative");
    }
    Ok(())
}

/// Validate lot source proportions sum to 100
pub fn validate_blend_proportions(proportions: &[Decimal]) -> Result<(), &'static str> {
    let total: Decimal = proportions.iter().sum();
    if total != Decimal::from(100) {
        return Err("Blend proportions must sum to 100%");
    }
    for p in proportions {
        if *p < Decimal::ZERO {
            return Err("Blend proportions cannot be negative");
        }
    }
    Ok(())
}

/// Validate cupping score is in valid range (SCA protocol)
pub fn validate_cupping_score(score: Decimal, is_full_range: bool) -> Result<(), &'static str> {
    let min = if is_full_range {
        Decimal::ZERO
    } else {
        Decimal::from(6)
    };
    let max = Decimal::from(10);

    if score < min || score > max {
        return Err("Cupping score out of valid range");
    }
    Ok(())
}

/// Validate defect counts and return grade classification
pub fn validate_and_classify_grade(defects: &DefectCount) -> GradeClassification {
    crate::models::classify_grade(defects)
}

/// Validate moisture content is in acceptable range (10-12% ideal for green beans)
pub fn validate_moisture_content(moisture: Decimal) -> Result<(), &'static str> {
    if moisture < Decimal::ZERO || moisture > Decimal::from(100) {
        return Err("Moisture content must be between 0 and 100%");
    }
    Ok(())
}

/// Check if moisture content is in ideal range for green beans
pub fn is_ideal_moisture(moisture: Decimal) -> bool {
    moisture >= Decimal::from(10) && moisture <= Decimal::from(12)
}

// ============================================================================
// General Validations
// ============================================================================

/// Validate email format (basic check)
pub fn validate_email(email: &str) -> Result<(), &'static str> {
    if email.contains('@') && email.contains('.') && email.len() >= 5 {
        Ok(())
    } else {
        Err("Invalid email format")
    }
}

/// Validate business code format (3-10 uppercase alphanumeric)
pub fn validate_business_code(code: &str) -> Result<(), &'static str> {
    if code.len() < 3 {
        return Err("Business code must be at least 3 characters");
    }
    if code.len() > 10 {
        return Err("Business code must be at most 10 characters");
    }
    if !code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
        return Err("Business code must be uppercase alphanumeric only");
    }
    Ok(())
}

/// Validate password strength
pub fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters");
    }
    Ok(())
}

// ============================================================================
// Thailand-Specific Validations
// ============================================================================

/// Validate Thai phone number format
/// Accepts: 0812345678, 081-234-5678, +66812345678
pub fn validate_thai_phone(phone: &str) -> Result<(), &'static str> {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    
    // Thai mobile: 10 digits starting with 0 (e.g., 0812345678)
    if digits.len() == 10 && digits.starts_with('0') {
        return Ok(());
    }
    // International format without leading 0: 9 digits (e.g., 812345678)
    if digits.len() == 9 && !digits.starts_with('0') {
        return Ok(());
    }
    // International format with country code: 11 digits starting with 66
    if digits.len() == 11 && digits.starts_with("66") {
        return Ok(());
    }
    
    Err("Invalid Thai phone number format")
}

/// Validate Thai National ID (เลขประจำตัวประชาชน)
/// 13-digit number with checksum validation
pub fn validate_thai_national_id(id: &str) -> Result<(), &'static str> {
    let digits: String = id.chars().filter(|c| c.is_ascii_digit()).collect();
    
    if digits.len() != 13 {
        return Err("Thai National ID must be 13 digits");
    }
    
    // Checksum validation using modulo 11 algorithm
    let chars: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if chars.len() != 13 {
        return Err("Invalid Thai National ID format");
    }
    
    let mut sum = 0;
    for (i, &digit) in chars.iter().take(12).enumerate() {
        sum += digit * (13 - i as u32);
    }
    
    let check_digit = (11 - (sum % 11)) % 10;
    if check_digit != chars[12] {
        return Err("Invalid Thai National ID checksum");
    }
    
    Ok(())
}

/// Validate Thai Tax ID (เลขประจำตัวผู้เสียภาษี)
/// 13-digit number for businesses/individuals
pub fn validate_thai_tax_id(tax_id: &str) -> Result<(), &'static str> {
    let digits: String = tax_id.chars().filter(|c| c.is_ascii_digit()).collect();
    
    if digits.len() != 13 {
        return Err("Thai Tax ID must be 13 digits");
    }
    
    // First digit indicates type: 0=individual, 1-9=juristic person
    let first_digit = digits.chars().next().unwrap();
    if !first_digit.is_ascii_digit() {
        return Err("Invalid Thai Tax ID format");
    }
    
    Ok(())
}

/// Thai provinces (จังหวัด) - coffee growing regions
pub const THAI_COFFEE_PROVINCES: &[&str] = &[
    "เชียงใหม่",      // Chiang Mai
    "เชียงราย",       // Chiang Rai
    "แม่ฮ่องสอน",     // Mae Hong Son
    "น่าน",           // Nan
    "พะเยา",          // Phayao
    "ลำปาง",          // Lampang
    "ลำพูน",          // Lamphun
    "แพร่",           // Phrae
    "ตาก",            // Tak
    "เพชรบูรณ์",      // Phetchabun
    "เลย",            // Loei
    "ชุมพร",          // Chumphon
    "ระนอง",          // Ranong
    "กระบี่",         // Krabi
    "สุราษฎร์ธานี",   // Surat Thani
    "นครศรีธรรมราช",  // Nakhon Si Thammarat
    "ยะลา",           // Yala
    "นราธิวาส",       // Narathiwat
];

/// Thai provinces in English
pub const THAI_COFFEE_PROVINCES_EN: &[&str] = &[
    "Chiang Mai",
    "Chiang Rai",
    "Mae Hong Son",
    "Nan",
    "Phayao",
    "Lampang",
    "Lamphun",
    "Phrae",
    "Tak",
    "Phetchabun",
    "Loei",
    "Chumphon",
    "Ranong",
    "Krabi",
    "Surat Thani",
    "Nakhon Si Thammarat",
    "Yala",
    "Narathiwat",
];

/// Validate province is a known Thai coffee-growing region
pub fn validate_thai_province(province: &str) -> Result<(), &'static str> {
    let province_lower = province.to_lowercase();
    
    // Check Thai names
    if THAI_COFFEE_PROVINCES.iter().any(|p| p.to_lowercase() == province_lower) {
        return Ok(());
    }
    
    // Check English names
    if THAI_COFFEE_PROVINCES_EN.iter().any(|p| p.to_lowercase() == province_lower) {
        return Ok(());
    }
    
    Err("Province is not a recognized Thai coffee-growing region")
}

/// Validate Thai GAP certificate number format
/// Format: GAP-YYYY-NNNNN (e.g., GAP-2024-00123)
pub fn validate_thai_gap_certificate(cert_number: &str) -> Result<(), &'static str> {
    let parts: Vec<&str> = cert_number.split('-').collect();
    
    if parts.len() != 3 {
        return Err("Thai GAP certificate must be in format GAP-YYYY-NNNNN");
    }
    
    if parts[0] != "GAP" {
        return Err("Thai GAP certificate must start with 'GAP'");
    }
    
    // Validate year
    if parts[1].len() != 4 || !parts[1].chars().all(|c| c.is_ascii_digit()) {
        return Err("Invalid year in Thai GAP certificate");
    }
    
    // Validate sequence number
    if parts[2].len() != 5 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return Err("Invalid sequence number in Thai GAP certificate");
    }
    
    Ok(())
}

/// Validate Organic Thailand certificate number format
/// Format: OT-YYYY-NNNNN
pub fn validate_organic_thailand_certificate(cert_number: &str) -> Result<(), &'static str> {
    let parts: Vec<&str> = cert_number.split('-').collect();
    
    if parts.len() != 3 {
        return Err("Organic Thailand certificate must be in format OT-YYYY-NNNNN");
    }
    
    if parts[0] != "OT" {
        return Err("Organic Thailand certificate must start with 'OT'");
    }
    
    // Validate year
    if parts[1].len() != 4 || !parts[1].chars().all(|c| c.is_ascii_digit()) {
        return Err("Invalid year in Organic Thailand certificate");
    }
    
    // Validate sequence number
    if parts[2].len() != 5 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return Err("Invalid sequence number in Organic Thailand certificate");
    }
    
    Ok(())
}

/// Validate altitude is reasonable for Thai coffee growing (typically 800-1800m)
pub fn validate_thai_coffee_altitude(altitude_meters: i32) -> Result<(), &'static str> {
    if altitude_meters < 0 {
        return Err("Altitude cannot be negative");
    }
    if altitude_meters > 3000 {
        return Err("Altitude exceeds maximum for Thailand");
    }
    Ok(())
}

/// Check if altitude is in optimal range for Thai Arabica coffee
pub fn is_optimal_arabica_altitude(altitude_meters: i32) -> bool {
    altitude_meters >= 800 && altitude_meters <= 1800
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Coffee Quality Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_ripeness_valid() {
        let ripeness = RipenessAssessment {
            underripe_percent: 10,
            ripe_percent: 80,
            overripe_percent: 10,
        };
        assert!(validate_ripeness(&ripeness).is_ok());
    }

    #[test]
    fn test_validate_ripeness_all_ripe() {
        let ripeness = RipenessAssessment {
            underripe_percent: 0,
            ripe_percent: 100,
            overripe_percent: 0,
        };
        assert!(validate_ripeness(&ripeness).is_ok());
    }

    #[test]
    fn test_validate_ripeness_invalid_sum() {
        let ripeness = RipenessAssessment {
            underripe_percent: 10,
            ripe_percent: 80,
            overripe_percent: 20,
        };
        assert!(validate_ripeness(&ripeness).is_err());
    }

    #[test]
    fn test_validate_blend_proportions_valid() {
        let proportions = vec![Decimal::from(60), Decimal::from(40)];
        assert!(validate_blend_proportions(&proportions).is_ok());
    }

    #[test]
    fn test_validate_blend_proportions_single_source() {
        let proportions = vec![Decimal::from(100)];
        assert!(validate_blend_proportions(&proportions).is_ok());
    }

    #[test]
    fn test_validate_blend_proportions_invalid() {
        let invalid = vec![Decimal::from(60), Decimal::from(50)];
        assert!(validate_blend_proportions(&invalid).is_err());
    }

    #[test]
    fn test_validate_cupping_score_valid() {
        assert!(validate_cupping_score(Decimal::from(8), false).is_ok());
        assert!(validate_cupping_score(Decimal::from(6), false).is_ok());
        assert!(validate_cupping_score(Decimal::from(10), false).is_ok());
    }

    #[test]
    fn test_validate_cupping_score_full_range() {
        assert!(validate_cupping_score(Decimal::from(0), true).is_ok());
        assert!(validate_cupping_score(Decimal::from(5), true).is_ok());
    }

    #[test]
    fn test_validate_cupping_score_invalid() {
        assert!(validate_cupping_score(Decimal::from(5), false).is_err());
        assert!(validate_cupping_score(Decimal::from(11), false).is_err());
    }

    #[test]
    fn test_moisture_content_validation() {
        assert!(validate_moisture_content(Decimal::from(11)).is_ok());
        assert!(validate_moisture_content(Decimal::from(0)).is_ok());
        assert!(validate_moisture_content(Decimal::from(100)).is_ok());
        assert!(validate_moisture_content(Decimal::from(-1)).is_err());
        assert!(validate_moisture_content(Decimal::from(101)).is_err());
    }

    #[test]
    fn test_ideal_moisture() {
        assert!(is_ideal_moisture(Decimal::from(10)));
        assert!(is_ideal_moisture(Decimal::from(11)));
        assert!(is_ideal_moisture(Decimal::from(12)));
        assert!(!is_ideal_moisture(Decimal::from(9)));
        assert!(!is_ideal_moisture(Decimal::from(13)));
    }

    // ========================================================================
    // General Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name@domain.co.th").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("no@domain").is_err());
        assert!(validate_email("@.").is_err());
    }

    #[test]
    fn test_validate_business_code_valid() {
        assert!(validate_business_code("DOI").is_ok());
        assert!(validate_business_code("CMI123").is_ok());
        assert!(validate_business_code("ABCDEFGHIJ").is_ok());
    }

    #[test]
    fn test_validate_business_code_invalid() {
        assert!(validate_business_code("AB").is_err()); // Too short
        assert!(validate_business_code("ABCDEFGHIJK").is_err()); // Too long
        assert!(validate_business_code("abc").is_err()); // Lowercase
        assert!(validate_business_code("AB-C").is_err()); // Special char
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("password123").is_ok());
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("short").is_err());
    }

    // ========================================================================
    // Thailand-Specific Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_thai_phone_valid() {
        // Standard Thai mobile
        assert!(validate_thai_phone("0812345678").is_ok());
        // With dashes
        assert!(validate_thai_phone("081-234-5678").is_ok());
        // Without leading zero
        assert!(validate_thai_phone("812345678").is_ok());
        // International format
        assert!(validate_thai_phone("+66812345678").is_ok());
        assert!(validate_thai_phone("66812345678").is_ok());
    }

    #[test]
    fn test_validate_thai_phone_invalid() {
        assert!(validate_thai_phone("12345").is_err());
        assert!(validate_thai_phone("123456789012").is_err());
        assert!(validate_thai_phone("abcdefghij").is_err());
    }

    #[test]
    fn test_validate_thai_national_id_valid() {
        // Valid Thai ID with correct checksum
        assert!(validate_thai_national_id("1100700000001").is_ok());
    }

    #[test]
    fn test_validate_thai_national_id_invalid() {
        // Wrong length
        assert!(validate_thai_national_id("123456789").is_err());
        // Invalid checksum
        assert!(validate_thai_national_id("1234567890123").is_err());
    }

    #[test]
    fn test_validate_thai_tax_id_valid() {
        assert!(validate_thai_tax_id("0123456789012").is_ok());
        assert!(validate_thai_tax_id("1234567890123").is_ok());
    }

    #[test]
    fn test_validate_thai_tax_id_invalid() {
        assert!(validate_thai_tax_id("123456789").is_err());
        assert!(validate_thai_tax_id("12345678901234").is_err());
    }

    #[test]
    fn test_validate_thai_province_valid() {
        // Thai names
        assert!(validate_thai_province("เชียงใหม่").is_ok());
        assert!(validate_thai_province("เชียงราย").is_ok());
        // English names
        assert!(validate_thai_province("Chiang Mai").is_ok());
        assert!(validate_thai_province("chiang rai").is_ok()); // Case insensitive
    }

    #[test]
    fn test_validate_thai_province_invalid() {
        assert!(validate_thai_province("Bangkok").is_err()); // Not a coffee region
        assert!(validate_thai_province("Unknown").is_err());
    }

    #[test]
    fn test_validate_thai_gap_certificate_valid() {
        assert!(validate_thai_gap_certificate("GAP-2024-00123").is_ok());
        assert!(validate_thai_gap_certificate("GAP-2023-99999").is_ok());
    }

    #[test]
    fn test_validate_thai_gap_certificate_invalid() {
        assert!(validate_thai_gap_certificate("GAP-24-123").is_err());
        assert!(validate_thai_gap_certificate("THAI-2024-00123").is_err());
        assert!(validate_thai_gap_certificate("GAP202400123").is_err());
    }

    #[test]
    fn test_validate_organic_thailand_certificate_valid() {
        assert!(validate_organic_thailand_certificate("OT-2024-00123").is_ok());
        assert!(validate_organic_thailand_certificate("OT-2023-99999").is_ok());
    }

    #[test]
    fn test_validate_organic_thailand_certificate_invalid() {
        assert!(validate_organic_thailand_certificate("OT-24-123").is_err());
        assert!(validate_organic_thailand_certificate("ORGANIC-2024-00123").is_err());
    }

    #[test]
    fn test_validate_thai_coffee_altitude_valid() {
        assert!(validate_thai_coffee_altitude(800).is_ok());
        assert!(validate_thai_coffee_altitude(1200).is_ok());
        assert!(validate_thai_coffee_altitude(2500).is_ok());
    }

    #[test]
    fn test_validate_thai_coffee_altitude_invalid() {
        assert!(validate_thai_coffee_altitude(-100).is_err());
        assert!(validate_thai_coffee_altitude(5000).is_err());
    }

    #[test]
    fn test_optimal_arabica_altitude() {
        assert!(is_optimal_arabica_altitude(800));
        assert!(is_optimal_arabica_altitude(1200));
        assert!(is_optimal_arabica_altitude(1800));
        assert!(!is_optimal_arabica_altitude(500));
        assert!(!is_optimal_arabica_altitude(2000));
    }
}
