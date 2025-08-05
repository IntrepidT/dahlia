use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: i64,
    pub code: String,
    pub school_name: String,
    pub invited_by_user_id: Option<i64>,
    pub role: String,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvitationRequest {
    pub school_name: String,
    pub role: String,
    pub max_uses: i32,
    pub expires_in_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationInfo {
    pub code: String,
    pub school_name: String,
    pub role: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub uses_remaining: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCode {
    pub id: i64,
    pub user_id: i64,
    pub code: String,
    pub verification_type: VerificationType,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationType {
    Email,
    Phone,
}

impl VerificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationType::Email => "email",
            VerificationType::Phone => "phone",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "email" => Ok(VerificationType::Email),
            "phone" => Ok(VerificationType::Phone),
            _ => Err(format!("Invalid verification type: {}", s)),
        }
    }
}

impl Invitation {
    pub fn is_valid(&self) -> bool {
        // Check if invitation hasn't expired
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        // Check if uses are available
        self.current_uses < self.max_uses
    }

    pub fn uses_remaining(&self) -> i32 {
        self.max_uses - self.current_uses
    }

    pub fn can_be_used(&self) -> bool {
        self.is_valid() && self.uses_remaining() > 0
    }
}

impl VerificationCode {
    pub fn is_valid(&self) -> bool {
        // Code hasn't been used and hasn't expired
        self.used_at.is_none() && Utc::now() < self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

// Helper for generating secure invitation codes
pub fn generate_invitation_code() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

// Helper for generating verification codes
pub fn generate_verification_code() -> String {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..999999))
}

// Phone number validation and normalization
pub fn normalize_phone_number(phone: &str) -> Result<String, String> {
    // Remove all non-digits
    let digits_only: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    
    // Handle US phone numbers
    match digits_only.len() {
        10 => {
            // 1234567890 -> +11234567890
            Ok(format!("+1{}", digits_only))
        }
        11 if digits_only.starts_with('1') => {
            // 11234567890 -> +11234567890
            Ok(format!("+{}", digits_only))
        }
        _ => Err("Phone number must be a valid US phone number (10 or 11 digits)".to_string()),
    }
}

pub fn format_phone_display(phone: &str) -> String {
    // Convert +11234567890 to (123) 456-7890
    if phone.starts_with("+1") && phone.len() == 12 {
        let digits = &phone[2..];
        format!("({}) {}-{}", 
            &digits[0..3], 
            &digits[3..6], 
            &digits[6..10]
        )
    } else {
        phone.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_normalization() {
        assert_eq!(normalize_phone_number("1234567890").unwrap(), "+11234567890");
        assert_eq!(normalize_phone_number("(123) 456-7890").unwrap(), "+11234567890");
        assert_eq!(normalize_phone_number("11234567890").unwrap(), "+11234567890");
        assert_eq!(normalize_phone_number("+1 123 456 7890").unwrap(), "+11234567890");
        
        assert!(normalize_phone_number("123456789").is_err()); // Too short
        assert!(normalize_phone_number("123456789012").is_err()); // Too long
    }

    #[test]
    fn test_phone_display() {
        assert_eq!(format_phone_display("+11234567890"), "(123) 456-7890");
    }

    #[test]
    fn test_invitation_validity() {
        let mut invitation = Invitation {
            id: 1,
            code: "TEST123".to_string(),
            school_name: "Test School".to_string(),
            invited_by_user_id: Some(1),
            role: "user".to_string(),
            max_uses: 5,
            current_uses: 2,
            expires_at: Some(Utc::now() + chrono::Duration::days(7)),
            created_at: Utc::now(),
        };

        assert!(invitation.is_valid());
        assert_eq!(invitation.uses_remaining(), 3);
        assert!(invitation.can_be_used());

        // Test expired invitation
        invitation.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!invitation.is_valid());
        assert!(!invitation.can_be_used());

        // Test exhausted uses
        invitation.expires_at = Some(Utc::now() + chrono::Duration::days(7));
        invitation.current_uses = 5;
        assert!(!invitation.can_be_used());
    }
}
