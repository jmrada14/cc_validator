//! REST API server for credit card validation.
//!
//! # Usage
//!
//! ```bash
//! # Start server
//! ccvalidator-server
//!
//! # With custom port
//! ccvalidator-server --port 8080
//! ```
//!
//! # Swagger UI
//!
//! Visit http://localhost:3000/swagger-ui/ for interactive API documentation.

use axum::{
    extract::Query,
    http::{header, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema, IntoParams};
use utoipa_swagger_ui::SwaggerUi;

use cc_validator::{
    validate, CardBrand,
    format, expiry, cvv, generate, detect,
};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Credit Card Validator API",
        version = "0.1.0",
        description = "Credit card validation REST API. Supports 14 card brands, CVV, expiry validation. Work in progress - no auth or rate limiting.",
        license(name = "MIT OR Apache-2.0"),
        contact(name = "API Support")
    ),
    tags(
        (name = "Validation", description = "Card number validation endpoints"),
        (name = "Detection", description = "Card brand detection from partial numbers"),
        (name = "Formatting", description = "Card number formatting utilities"),
        (name = "Generation", description = "Test card number generation"),
        (name = "CVV", description = "CVV/CVC/CID validation"),
        (name = "Expiry", description = "Expiry date validation"),
        (name = "System", description = "Health and status endpoints")
    ),
    paths(
        validate_card,
        validate_batch,
        detect_brand_handler,
        format_card,
        generate_cards,
        validate_cvv_handler,
        validate_expiry_handler,
        health,
    ),
    components(schemas(
        ValidateRequest,
        ValidateResponse,
        BatchValidateRequest,
        BatchValidateResponse,
        BatchSummary,
        DetectQuery,
        DetectResponse,
        FormatRequest,
        FormatResponse,
        GenerateRequest,
        GenerateResponse,
        CvvRequest,
        CvvResponse,
        ExpiryRequest,
        ExpiryResponse,
        HealthResponse,
    ))
)]
struct ApiDoc;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"card_number": "4111-1111-1111-1111"}))]
struct ValidateRequest {
    /// Card number to validate. Accepts digits with optional spaces or dashes as separators.
    /// Example formats: "4111111111111111", "4111-1111-1111-1111", "4111 1111 1111 1111"
    card_number: String,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "valid": true,
    "brand": "Visa",
    "last_four": "1111",
    "masked": "****-****-****-1111"
}))]
struct ValidateResponse {
    /// Whether the card number passed all validation checks (Luhn checksum, length, brand detection)
    valid: bool,
    /// Detected card brand. Supports: Visa, Mastercard, American Express, Discover, Diners Club, JCB, UnionPay, Maestro, Mir, RuPay, Verve, Elo, Troy, BC Card
    #[serde(skip_serializing_if = "Option::is_none")]
    brand: Option<String>,
    /// Last 4 digits of the card number (safe for display per PCI-DSS)
    #[serde(skip_serializing_if = "Option::is_none")]
    last_four: Option<String>,
    /// Masked card number in format ****-****-****-1234 (safe for logging and display)
    #[serde(skip_serializing_if = "Option::is_none")]
    masked: Option<String>,
    /// Human-readable error message explaining why validation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"card_numbers": ["4111111111111111", "5500000000000004", "378282246310005"]}))]
struct BatchValidateRequest {
    /// List of card numbers to validate. Each number can include spaces or dashes.
    card_numbers: Vec<String>,
}

#[derive(Serialize, ToSchema)]
struct BatchValidateResponse {
    /// Validation results for each card
    results: Vec<ValidateResponse>,
    /// Summary statistics
    summary: BatchSummary,
}

#[derive(Serialize, ToSchema)]
struct BatchSummary {
    /// Total cards processed
    total: usize,
    /// Number of valid cards
    valid: usize,
    /// Number of invalid cards
    invalid: usize,
}

#[derive(Deserialize, ToSchema, IntoParams)]
struct DetectQuery {
    /// Card number or prefix to detect
    card: String,
}

#[derive(Serialize, ToSchema)]
struct DetectResponse {
    /// Detected brand name
    brand: Option<String>,
    /// Valid lengths for this brand
    valid_lengths: Option<Vec<usize>>,
}

#[derive(Deserialize, ToSchema)]
struct FormatRequest {
    /// Card number to format
    card_number: String,
    /// Separator character (default: space)
    #[serde(default = "default_separator")]
    separator: String,
}

fn default_separator() -> String {
    " ".to_string()
}

#[derive(Serialize, ToSchema)]
struct FormatResponse {
    /// Formatted card number
    formatted: String,
    /// Card number with formatting stripped
    stripped: String,
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"brand": "visa", "count": 3, "formatted": true}))]
struct GenerateRequest {
    /// Card brand to generate. Options: visa, mastercard, amex, discover, jcb, diners, unionpay, maestro, mir, rupay, verve, elo, troy, bccard
    brand: String,
    /// Number of test cards to generate (1-100, default: 1)
    #[serde(default = "default_count")]
    count: usize,
    /// Whether to format output with spaces (e.g., "4111 1111 1111 1111")
    #[serde(default)]
    formatted: bool,
}

fn default_count() -> usize {
    1
}

#[derive(Serialize, ToSchema)]
struct GenerateResponse {
    /// Generated card numbers
    cards: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"cvv": "123", "brand": "visa"}))]
struct CvvRequest {
    /// CVV/CVC/CID code to validate (3-4 digits)
    cvv: String,
    /// Card brand for brand-specific validation. American Express requires 4 digits, all others require 3.
    #[serde(skip_serializing_if = "Option::is_none")]
    brand: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct CvvResponse {
    /// Whether the CVV is valid
    valid: bool,
    /// CVV length
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<usize>,
    /// Error message if validation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"date": "12/25"}))]
struct ExpiryRequest {
    /// Expiry date in various formats: MM/YY, MM/YYYY, MMYY, MMYYYY, MM-YY, MM-YYYY
    date: String,
}

#[derive(Serialize, ToSchema)]
struct ExpiryResponse {
    /// Whether the expiry is valid
    valid: bool,
    /// Month (1-12)
    #[serde(skip_serializing_if = "Option::is_none")]
    month: Option<u8>,
    /// Year (4 digits)
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<u16>,
    /// Whether the card is expired
    #[serde(skip_serializing_if = "Option::is_none")]
    expired: Option<bool>,
    /// Formatted date (MM/YY)
    #[serde(skip_serializing_if = "Option::is_none")]
    formatted: Option<String>,
    /// Error message if validation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    /// Service status
    status: String,
    /// API version
    version: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Validate a credit card number
#[utoipa::path(
    post,
    path = "/validate",
    request_body = ValidateRequest,
    responses(
        (status = 200, description = "Validation result", body = ValidateResponse)
    ),
    tag = "Validation"
)]
async fn validate_card(Json(req): Json<ValidateRequest>) -> Json<ValidateResponse> {
    match validate(&req.card_number) {
        Ok(card) => Json(ValidateResponse {
            valid: true,
            brand: Some(card.brand().name().to_string()),
            last_four: Some(card.last_four().to_string()),
            masked: Some(card.masked()),
            error: None,
        }),
        Err(e) => Json(ValidateResponse {
            valid: false,
            brand: None,
            last_four: None,
            masked: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Validate multiple card numbers
#[utoipa::path(
    post,
    path = "/validate/batch",
    request_body = BatchValidateRequest,
    responses(
        (status = 200, description = "Batch validation results", body = BatchValidateResponse)
    ),
    tag = "Validation"
)]
async fn validate_batch(Json(req): Json<BatchValidateRequest>) -> Json<BatchValidateResponse> {
    let results: Vec<ValidateResponse> = req
        .card_numbers
        .iter()
        .map(|card| match validate(card) {
            Ok(c) => ValidateResponse {
                valid: true,
                brand: Some(c.brand().name().to_string()),
                last_four: Some(c.last_four().to_string()),
                masked: Some(c.masked()),
                error: None,
            },
            Err(e) => ValidateResponse {
                valid: false,
                brand: None,
                last_four: None,
                masked: None,
                error: Some(e.to_string()),
            },
        })
        .collect();

    let valid_count = results.iter().filter(|r| r.valid).count();

    Json(BatchValidateResponse {
        summary: BatchSummary {
            total: results.len(),
            valid: valid_count,
            invalid: results.len() - valid_count,
        },
        results,
    })
}

/// Detect card brand from number
#[utoipa::path(
    get,
    path = "/detect",
    params(DetectQuery),
    responses(
        (status = 200, description = "Detected brand", body = DetectResponse)
    ),
    tag = "Detection"
)]
async fn detect_brand_handler(Query(query): Query<DetectQuery>) -> Json<DetectResponse> {
    let digits: Vec<u8> = query
        .card
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect();

    match detect::detect_brand(&digits) {
        Some(brand) => Json(DetectResponse {
            brand: Some(brand.name().to_string()),
            valid_lengths: Some(brand.valid_lengths().iter().map(|&l| l as usize).collect()),
        }),
        None => Json(DetectResponse {
            brand: None,
            valid_lengths: None,
        }),
    }
}

/// Format a card number
#[utoipa::path(
    post,
    path = "/format",
    request_body = FormatRequest,
    responses(
        (status = 200, description = "Formatted card", body = FormatResponse)
    ),
    tag = "Formatting"
)]
async fn format_card(Json(req): Json<FormatRequest>) -> Json<FormatResponse> {
    Json(FormatResponse {
        formatted: format::format_with_separator(&req.card_number, &req.separator),
        stripped: format::strip_formatting(&req.card_number),
    })
}

/// Generate test card numbers
#[utoipa::path(
    post,
    path = "/generate",
    request_body = GenerateRequest,
    responses(
        (status = 200, description = "Generated cards", body = GenerateResponse),
        (status = 400, description = "Invalid brand")
    ),
    tag = "Generation"
)]
async fn generate_cards(Json(req): Json<GenerateRequest>) -> Result<Json<GenerateResponse>, (StatusCode, String)> {
    let brand = parse_brand(&req.brand)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("Unknown brand: {}", req.brand)))?;

    let count = req.count.min(100); // Limit to 100 cards

    let cards: Vec<String> = (0..count)
        .map(|_| {
            let card = generate::generate_card(brand);
            if req.formatted {
                format::format_card_number(&card)
            } else {
                card
            }
        })
        .collect();

    Ok(Json(GenerateResponse { cards }))
}

/// Validate a CVV/CVC code
#[utoipa::path(
    post,
    path = "/cvv/validate",
    request_body = CvvRequest,
    responses(
        (status = 200, description = "CVV validation result", body = CvvResponse)
    ),
    tag = "CVV"
)]
async fn validate_cvv_handler(Json(req): Json<CvvRequest>) -> Json<CvvResponse> {
    let result = if let Some(brand_str) = &req.brand {
        if let Some(brand) = parse_brand(brand_str) {
            cvv::validate_cvv_for_brand(&req.cvv, brand)
        } else {
            return Json(CvvResponse {
                valid: false,
                length: None,
                error: Some(format!("Unknown brand: {}", brand_str)),
            });
        }
    } else {
        cvv::validate_cvv(&req.cvv)
    };

    match result {
        Ok(validated) => Json(CvvResponse {
            valid: true,
            length: Some(validated.length()),
            error: None,
        }),
        Err(e) => Json(CvvResponse {
            valid: false,
            length: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Validate an expiry date
#[utoipa::path(
    post,
    path = "/expiry/validate",
    request_body = ExpiryRequest,
    responses(
        (status = 200, description = "Expiry validation result", body = ExpiryResponse)
    ),
    tag = "Expiry"
)]
async fn validate_expiry_handler(Json(req): Json<ExpiryRequest>) -> Json<ExpiryResponse> {
    match expiry::validate_expiry(&req.date) {
        Ok(exp) => Json(ExpiryResponse {
            valid: true,
            month: Some(exp.month()),
            year: Some(exp.year()),
            expired: Some(exp.is_expired()),
            formatted: Some(exp.format_short()),
            error: None,
        }),
        Err(e) => Json(ExpiryResponse {
            valid: false,
            month: None,
            year: None,
            expired: None,
            formatted: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Health check
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "System"
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ============================================================================
// Helpers
// ============================================================================

fn parse_brand(s: &str) -> Option<CardBrand> {
    match s.to_lowercase().as_str() {
        "visa" => Some(CardBrand::Visa),
        "mastercard" | "mc" => Some(CardBrand::Mastercard),
        "amex" | "american express" => Some(CardBrand::Amex),
        "discover" => Some(CardBrand::Discover),
        "jcb" => Some(CardBrand::Jcb),
        "diners" | "dinersclub" | "diners club" => Some(CardBrand::DinersClub),
        "unionpay" | "union pay" => Some(CardBrand::UnionPay),
        "maestro" => Some(CardBrand::Maestro),
        "mir" => Some(CardBrand::Mir),
        "rupay" => Some(CardBrand::RuPay),
        "verve" => Some(CardBrand::Verve),
        "elo" => Some(CardBrand::Elo),
        "troy" => Some(CardBrand::Troy),
        "bccard" | "bc card" => Some(CardBrand::BcCard),
        _ => None,
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse args
    let port: u16 = std::env::args()
        .skip_while(|a| a != "--port")
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
        .allow_origin(Any);

    // Build router with Swagger UI
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/validate", post(validate_card))
        .route("/validate/batch", post(validate_batch))
        .route("/detect", get(detect_brand_handler))
        .route("/format", post(format_card))
        .route("/generate", post(generate_cards))
        .route("/cvv/validate", post(validate_cvv_handler))
        .route("/expiry/validate", post(validate_expiry_handler))
        .route("/health", get(health))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}:{}/swagger-ui/", "localhost", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
