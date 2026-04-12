// Cognition module for task planning, decision making, and cognitive processing
// This module handles communication with the Gemini LLM API

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CognitionError {
    #[error("API key not found. Please set your Gemini API key.")]
    ApiKeyNotFound,
    
    #[error("Failed to retrieve API key from keyring: {0}")]
    KeyringError(#[from] keyring::Error),
    
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("API returned an error: {0}")]
    ApiError(String),
    
    #[error("Failed to parse API response: {0}")]
    ParseError(String),
}

pub struct Cognition;

impl Cognition {
    pub fn new() -> Self {
        Self
    }
}

// Request/Response structures for Gemini API
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    #[serde(default)]
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Debug, Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Debug, Deserialize)]
struct PartResponse {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    message: String,
    code: Option<i32>,
}

/// Retrieves the Gemini API key from the system keyring
pub fn get_api_key() -> Result<String, CognitionError> {
    let service = "nyx";
    let username = "gemini_api_key";
    
    let entry = keyring::Entry::new(service, username)?;
    entry.get_password().map_err(|e| {
        if matches!(e, keyring::Error::NoEntry) {
            CognitionError::ApiKeyNotFound
        } else {
            CognitionError::KeyringError(e)
        }
    })
}

/// Stores the Gemini API key in the system keyring
pub fn set_api_key(api_key: &str) -> Result<(), CognitionError> {
    let service = "nyx";
    let username = "gemini_api_key";
    
    let entry = keyring::Entry::new(service, username)?;
    entry.set_password(api_key)?;
    Ok(())
}

/// Sends a prompt to the Gemini API and returns the response
pub async fn ask_gemini(prompt: &str) -> Result<String, CognitionError> {
    // 1. Get API key from keyring
    let api_key = get_api_key()?;

    // 2. Create HTTP client
    let client = reqwest::Client::new();

    // 3. Build request body
    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: prompt.to_string(),
            }],
        }],
    };

    // 4. Send the POST request with header-based authentication
    let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";
    
    let res = client
        .post(url)
        .header("x-goog-api-key", api_key) // Use header for auth
        .json(&request_body)
        .send()
        .await?;

    // 5. Await and handle potential HTTP errors
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CognitionError::ApiError(format!(
            "HTTP {}: {}",
            status, error_text
        )));
    }

    // 6. Parse the response
    let response: GeminiResponse = res.json().await.map_err(|e| {
        CognitionError::ParseError(format!("Failed to parse JSON response: {}", e))
    })?;

    // 7. Check for API-level errors
    if let Some(error) = response.error {
        return Err(CognitionError::ApiError(format!(
            "Gemini API error: {} (code: {:?})",
            error.message,
            error.code
        )));
    }

    // 8. Extract the text from the response
    if let Some(candidates) = response.candidates {
        if let Some(candidate) = candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
    }

    Err(CognitionError::ParseError(
        "No text content found in API response".to_string(),
    ))
}
