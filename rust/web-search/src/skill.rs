//! web-search-skill — web search via Tavily API.
//!
//! @skill:id      ai.luminarys.rust.web-search
//! @skill:name    "Web Search"
//! @skill:version 1.0.0
//! @skill:desc    "Performs web search. Returns structured JSON results with advanced depth."
//!
//! Build:
//!   lmsk generate -lang rust -out src .
//!   cargo build --target wasm32-wasip1 --release

use luminarys_sdk::prelude::*;

// ── Skill logic ───────────────────────────────────────────────────────────────

/// Execute a web search query via Tavily API.
///
/// @skill:method search "Search the web and return JSON results."
/// @skill:param  query required "Search query string"
/// @skill:result "JSON string containing search results"
pub fn search(_ctx: &mut Context, query: String) -> Result<String, SkillError> {
    let api_key = get_env("TAVILY_API_KEY");
    if api_key.is_empty() {
        return Err(SkillError("TAVILY_API_KEY environment variable not set".into()));
    }

    let body = serde_json::json!({
        "query": query,
        "search_depth": "advanced",
        "include_favicon": false,
    });

    let resp = http_request(HttpRequestOptions {
        method: "POST".into(),
        url: "https://api.tavily.com/search".into(),
        headers: vec![
            Header { name: "Authorization".into(), value: format!("Bearer {api_key}") },
            Header { name: "Content-Type".into(), value: "application/json".into() },
        ],
        body: body.to_string().into_bytes(),
        ..Default::default()
    })?;

    Ok(String::from_utf8_lossy(&resp.body).into_owned())
}
