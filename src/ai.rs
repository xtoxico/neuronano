use reqwest::Client;
use serde_json::{json, Value};
use anyhow::{Result, anyhow};
use log::{info, error, debug};

const GEMINI_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-latest:generateContent";

pub async fn request_gemini(api_key: String, current_code: String, filename: String, user_instruction: String) -> Result<String> {
    let client = Client::new();
    
    info!("Preparing Gemini API request for file: {}", filename);

    let system_prompt = format!(
        "You are an intelligent text editor engine. I will provide a file named \"{}\" with the following content. The user wants to: \"{}\". RULES:

Return ONLY the fully updated file content. No markdown code blocks. No conversational text.

If the user asks for explanations, insert them as COMMENTS inside the code (using correct syntax for {}).

Preserve indentation.",
        filename, user_instruction, filename
    );

    let body = json!({
        "contents": [{
            "parts": [{
                "text": format!("{}\n\nCODE:\n{}", system_prompt, current_code)
            }]
        }]
    });

    debug!("Payload: {}", body);

    let url = format!("{}?key={}", GEMINI_URL, api_key);
    info!("Sending request to Gemini Flash Latest...");

    let response = client.post(&url)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("API Error: Status {}, Body: {}", status, error_text);
        return Err(anyhow!("Gemini API Error {}: {}", status, error_text));
    }

    info!("Gemini API request successful.");

    let json_resp: Value = response.json().await?;
    
    let text = json_resp["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| {
            error!("Invalid API response structure: {:?}", json_resp);
            anyhow!("Invalid API response structure")
        })?
        .to_string();

    Ok(clean_markdown(&text))
}

fn clean_markdown(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    
    if lines.is_empty() {
        return String::new();
    }

    // Remove first line if it starts with ```
    if let Some(first) = lines.first() {
        if first.trim().starts_with("```") {
            lines.remove(0);
        }
    }

    // Remove last line if it starts with ```
    if let Some(last) = lines.last() {
        if last.trim().starts_with("```") {
            lines.pop();
        }
    }

    lines.join("\n")
}
