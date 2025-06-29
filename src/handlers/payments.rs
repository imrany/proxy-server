// Payment-related routes
use std::{collections::HashMap, sync::Arc};
use hyper::StatusCode;
use serde_json::{Value, json};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use axum::response::Json;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub client_ip: String,
    pub amount: f64, // KSH amount
    pub payment_date: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
    pub is_active: bool,
    pub payment_method: String, // "mpesa", "bank", "cash", etc.
    pub transaction_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub client_ip: String,
    pub amount: f64,
    pub payment_method: String,
    pub transaction_id: Option<String>,
}

pub type PaymentState = Arc<RwLock<HashMap<String, PaymentRecord>>>;
const MONTHLY_FEE: f64 = 150.0; // KSH 150

pub async fn record_payment(
    axum::extract::State(payments): axum::extract::State<PaymentState>,
    Json(payment_req): Json<PaymentRequest>,
) -> Result<Json<Value>, StatusCode> {
    if payment_req.amount < MONTHLY_FEE {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let payment_record = PaymentRecord {
        client_ip: payment_req.client_ip.clone(),
        amount: payment_req.amount,
        payment_date: Utc::now(),
        expiry_date: Utc::now() + Duration::days(30), // 30 days validity
        is_active: true,
        payment_method: payment_req.payment_method,
        transaction_id: payment_req.transaction_id,
    };
    
    let mut payment_records = payments.write().await;
    payment_records.insert(payment_req.client_ip.clone(), payment_record.clone());
    
    tracing::info!("💰 Payment recorded for IP: {} | Amount: KSH {}", 
        payment_req.client_ip, payment_req.amount);
    
    Ok(Json(json!({
        "status": "success",
        "message": "Payment recorded successfully",
        "expiry_date": payment_record.expiry_date,
        "valid_until": payment_record.expiry_date.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    })))
}

pub async fn get_all_payments(
    axum::extract::State(payments): axum::extract::State<PaymentState>
) -> Json<Value> {
    let payment_records = payments.read().await;
    let mut active_payments = Vec::new();
    let mut expired_payments = Vec::new();
    
    for (ip, payment) in payment_records.iter() {
        let is_valid = payment.is_active && payment.expiry_date > Utc::now();
        let payment_info = json!({
            "client_ip": ip,
            "amount": payment.amount,
            "payment_date": payment.payment_date,
            "expiry_date": payment.expiry_date,
            "payment_method": payment.payment_method,
            "transaction_id": payment.transaction_id,
            "days_remaining": payment.expiry_date.signed_duration_since(Utc::now()).num_days()
        });
        
        if is_valid {
            active_payments.push(payment_info);
        } else {
            expired_payments.push(payment_info);
        }
    }
    
    Json(json!({
        "active_payments": active_payments,
        "expired_payments": expired_payments,
        "total_active": active_payments.len(),
        "total_expired": expired_payments.len()
    }))
}

pub async fn check_payment_status(
    axum::extract::State(payments): axum::extract::State<PaymentState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Json<Value> {
    if let Some(client_ip) = params.get("ip") {
        let payment_records = payments.read().await;
        
        if let Some(payment) = payment_records.get(client_ip) {
            let is_valid = payment.is_active && payment.expiry_date > Utc::now();
            let days_remaining = if is_valid {
                payment.expiry_date.signed_duration_since(Utc::now()).num_days()
            } else {
                0
            };
            
            Json(json!({
                "client_ip": client_ip,
                "has_valid_payment": is_valid,
                "payment_date": payment.payment_date,
                "expiry_date": payment.expiry_date,
                "days_remaining": days_remaining,
                "amount_paid": payment.amount,
                "payment_method": payment.payment_method
            }))
        } else {
            Json(json!({
                "client_ip": client_ip,
                "has_valid_payment": false,
                "message": "No payment record found"
            }))
        }
    } else {
        Json(json!({
            "error": "IP parameter required"
        }))
    }
}

// Helper function to check if client has valid payment
pub async fn has_valid_payment(payments: &PaymentState, client_ip: &str) -> bool {
    let payment_records = payments.read().await;
    
    if let Some(payment) = payment_records.get(client_ip) {
        payment.is_active && payment.expiry_date > Utc::now()
    } else {
        false
    }
}
