use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};
use std::collections::HashMap;
use tokio;

#[derive(Debug, Serialize, Deserialize)]
struct ProofRequest {
    proof_type: String,
    parameters: HashMap<String, serde_json::Value>,
    user_agent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProofResponse {
    verified: bool,
    proof_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    cost_saved: f64,
    time_saved: f64,
    metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ROICalculation {
    passengers_per_year: u64,
    traditional_cost: f64,
    proofzk_cost: f64,
    annual_savings: f64,
    savings_percentage: f64,
    break_even_days: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DemoStats {
    total_demos: u64,
    age_verifications: u64,
    passport_verifications: u64,
    total_cost_saved: f64,
    average_time_saved: f64,
    airlines_tested: Vec<String>,
}

/// Main API handler for proof verification
async fn verify_proof(proof_req: ProofRequest) -> Result<impl Reply, Rejection> {
    println!("🔐 Proof verification request: {:?}", proof_req.proof_type);
    
    let response = match proof_req.proof_type.as_str() {
        "age_verification" => {
            let min_age = proof_req.parameters.get("minimum_age")
                .and_then(|v| v.as_u64())
                .unwrap_or(18) as u8;
            
            handle_age_verification(min_age).await
        },
        "passport_verification" => {
            handle_passport_verification().await
        },
        _ => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Unsupported proof type",
                    "supported_types": ["age_verification", "passport_verification"]
                })),
                warp::http::StatusCode::BAD_REQUEST
            ));
        }
    };
    
    // Log demo usage for analytics
    log_demo_usage(&proof_req).await;
    
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::OK
    ))
}

async fn handle_age_verification(min_age: u8) -> ProofResponse {
    // Simulate zero-knowledge proof verification
    let verified = true; // In demo, always verify successfully
    let proof_id = format!("age_proof_{}", uuid::Uuid::new_v4());
    
    let mut metadata = HashMap::new();
    metadata.insert("min_age".to_string(), serde_json::Value::Number(min_age.into()));
    metadata.insert("actual_age_revealed".to_string(), serde_json::Value::Bool(false));
    metadata.insert("verification_method".to_string(), serde_json::Value::String("biometric".to_string()));
    
    ProofResponse {
        verified,
        proof_id,
        timestamp: chrono::Utc::now(),
        cost_saved: 14.50, // $15 traditional - $0.50 ProofZK
        time_saved: 2.8,   // minutes saved vs traditional
        metadata,
    }
}

async fn handle_passport_verification() -> ProofResponse {
    // Simulate passport DID verification
    let verified = true;
    let proof_id = format!("passport_proof_{}", uuid::Uuid::new_v4());
    
    let mut metadata = HashMap::new();
    metadata.insert("document_type".to_string(), serde_json::Value::String("passport".to_string()));
    metadata.insert("nationality_revealed".to_string(), serde_json::Value::Bool(false));
    metadata.insert("passport_data_stored".to_string(), serde_json::Value::Bool(false));
    metadata.insert("government_did_verified".to_string(), serde_json::Value::Bool(true));
    
    ProofResponse {
        verified,
        proof_id,
        timestamp: chrono::Utc::now(),
        cost_saved: 24.25, // $25 traditional - $0.75 ProofZK
        time_saved: 4.5,   // minutes saved vs traditional
        metadata,
    }
}

/// Calculate ROI for airlines
async fn calculate_roi(passengers: u64) -> Result<impl Reply, Rejection> {
    let traditional_cost_per_verification = 15.0;
    let proofzk_cost_per_verification = 0.5;
    
    let traditional_cost = passengers as f64 * traditional_cost_per_verification;
    let proofzk_cost = passengers as f64 * proofzk_cost_per_verification;
    let annual_savings = traditional_cost - proofzk_cost;
    let savings_percentage = (annual_savings / traditional_cost) * 100.0;
    
    // Calculate break-even time (how many days to recover implementation cost)
    let implementation_cost = 50000.0; // Estimated implementation cost
    let daily_savings = annual_savings / 365.0;
    let break_even_days = implementation_cost / daily_savings;
    
    let roi = ROICalculation {
        passengers_per_year: passengers,
        traditional_cost,
        proofzk_cost,
        annual_savings,
        savings_percentage,
        break_even_days,
    };
    
    Ok(warp::reply::json(&roi))
}

/// Get demo statistics for partnerships
async fn get_demo_stats() -> Result<impl Reply, Rejection> {
    // In production, this would query a real database
    let stats = DemoStats {
        total_demos: 1247,
        age_verifications: 892,
        passport_verifications: 355,
        total_cost_saved: 18235.50,
        average_time_saved: 3.2,
        airlines_tested: vec![
            "Demo Airline".to_string(),
            "Test Airways".to_string(),
            "Partnership Evaluation".to_string(),
        ],
    };
    
    Ok(warp::reply::json(&stats))
}

/// Health check endpoint
async fn health_check() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status(
        "ProofZK API is healthy",
        warp::http::StatusCode::OK
    ))
}

/// Log demo usage for analytics
async fn log_demo_usage(proof_req: &ProofRequest) {
    let timestamp = chrono::Utc::now();
    let user_agent = proof_req.user_agent.as_deref().unwrap_or("unknown");
    
    println!("📊 Demo usage: {} at {} from {}", 
             proof_req.proof_type, timestamp, user_agent);
    
    // In production, save to database or analytics service
}

/// CORS filter for demo
fn with_cors() -> impl Filter<Extract = (), Error = std::convert::Infallible> + Clone {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
}

#[tokio::main]
async fn main() {
    println!("🚀 Starting ProofZK Demo API Server...");
    
    // Configure logging
    env_logger::init();
    
    // API Routes
    let verify = warp::path("api")
        .and(warp::path("verify"))
        .and(warp::post())
        .and(warp::body::json())
        .and_then(verify_proof);
    
    let roi = warp::path("api")
        .and(warp::path("roi"))
        .and(warp::path::param::<u64>())
        .and(warp::get())
        .and_then(calculate_roi);
    
    let stats = warp::path("api")
        .and(warp::path("stats"))
        .and(warp::get())
        .and_then(get_demo_stats);
    
    let health = warp::path("health")
        .and(warp::get())
        .and_then(health_check);
    
    // Combine all routes
    let routes = verify
        .or(roi)
        .or(stats)
        .or(health)
        .with(with_cors())
        .with(warp::log("proofzk_api"));
    
    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid PORT environment variable");
    
    println!("🎯 ProofZK API listening on http://127.0.0.1:{}", port);
    println!("📚 API Endpoints:");
    println!("   POST /api/verify - Verify proofs");
    println!("   GET  /api/roi/<passengers> - Calculate ROI");
    println!("   GET  /api/stats - Demo statistics");
    println!("   GET  /health - Health check");
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;
}