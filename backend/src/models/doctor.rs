use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Doctor {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub specialization: String,
    pub department: Option<String>,
    pub consultation_fee: Decimal,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDoctor {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    #[validate(length(min = 1, max = 120))]
    pub specialization: String,
    pub department: Option<String>,
    pub consultation_fee: Option<Decimal>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 32))]
    pub phone: Option<String>,
}
