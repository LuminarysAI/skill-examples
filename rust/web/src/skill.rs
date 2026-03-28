//! web-skill — outbound HTTP/HTTPS requests.
//!
//! @skill:id      ai.luminarys.rust.web
//! @skill:name    "Web Skill"
//! @skill:version 1.0.0
//! @skill:desc    "Outbound HTTP and HTTPS requests: GET, POST, and custom requests with headers."
//!
//! Build:
//!   lmsk generate -lang rust -out src .
//!   cargo build --target wasm32-wasip1 --release

use luminarys_sdk::prelude::*;
use serde_json::Value;

// ── Skill logic ───────────────────────────────────────────────────────────────

/// Perform an HTTP GET request and return the response body as text.
///
/// @skill:method get "Perform an HTTP GET request and return the response body."
/// @skill:param  url        required "Full URL to fetch (must match allowlist)" example:https://api.example.com/data
/// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30s default" default:0
/// @skill:result "Response body as UTF-8 text"
pub fn get(_ctx: &mut Context, url: String, timeout_ms: i64) -> Result<String, SkillError> {
    let resp = http_get(&url, timeout_ms, 0)?;
    if resp.status >= 400 {
        return Err(SkillError(format!(
            "HTTP {}: {}",
            resp.status,
            String::from_utf8_lossy(&resp.body)
        )));
    }
    Ok(String::from_utf8_lossy(&resp.body).into_owned())
}

/// Fetch a URL and return the parsed JSON response as pretty-printed text.
///
/// @skill:method get_json "Fetch a URL and return the parsed JSON response."
/// @skill:param  url        required "Full URL to fetch (must return JSON)"
/// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30s default" default:0
/// @skill:result "Pretty-printed JSON response"
pub fn get_json(_ctx: &mut Context, url: String, timeout_ms: i64) -> Result<String, SkillError> {
    let resp = http_get(&url, timeout_ms, 0)?;
    if resp.status >= 400 {
        return Err(SkillError(format!("HTTP {}", resp.status)));
    }
    match serde_json::from_slice::<Value>(&resp.body) {
        Ok(v) => Ok(serde_json::to_string_pretty(&v)
            .unwrap_or_else(|_| String::from_utf8_lossy(&resp.body).into_owned())),
        Err(_) => Ok(String::from_utf8_lossy(&resp.body).into_owned()),
    }
}

/// Perform an HTTP POST with a JSON body.
///
/// @skill:method post "Perform an HTTP POST request with a JSON body."
/// @skill:param  url        required "Target URL (must match allowlist)"
/// @skill:param  body       required "JSON string to send as the request body"
/// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30s default" default:0
/// @skill:result "Response body as text"
pub fn post(_ctx: &mut Context, url: String, body: String, timeout_ms: i64) -> Result<String, SkillError> {
    let resp = http_post(&url, body.into_bytes(), "application/json", timeout_ms, 0)?;
    if resp.status >= 400 {
        return Err(SkillError(format!(
            "HTTP {}: {}",
            resp.status,
            String::from_utf8_lossy(&resp.body)
        )));
    }
    Ok(String::from_utf8_lossy(&resp.body).into_owned())
}

/// Perform a fully customised HTTP request with explicit headers.
///
/// @skill:method request "Perform a custom HTTP request with explicit method and headers."
/// @skill:param  method     required "HTTP method: GET, POST, PUT, PATCH, DELETE" example:POST
/// @skill:param  url        required "Target URL (must match allowlist)"
/// @skill:param  body       optional "Request body as text or JSON string" default:""
/// @skill:param  headers    optional "Headers as JSON-encoded string, e.g. {\"Content-Type\":\"application/json\"}" default:""
/// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30s default" default:0
/// @skill:result "JSON with status and body"
pub fn request(
    _ctx: &mut Context,
    method: String,
    url: String,
    body: String,
    headers_json: String,
    timeout_ms: i64,
) -> Result<String, SkillError> {
    let hdrs = headers_from_json(&headers_json);

    let resp = http_request(HttpRequestOptions {
        method,
        url,
        headers: hdrs,
        body: body.into_bytes(),
        timeout_ms,
        follow_redirects: true,
        ..Default::default()
    })?;

    let result = serde_json::json!({
        "status": resp.status,
        "body": String::from_utf8_lossy(&resp.body),
    });
    Ok(result.to_string())
}

/// Perform an HTTP HEAD request and return status and headers.
///
/// @skill:method head "Perform an HTTP HEAD request and return status and headers."
/// @skill:param  url        required "Target URL (must match allowlist)"
/// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30s default" default:0
/// @skill:result "JSON with status code and response headers"
pub fn head(_ctx: &mut Context, url: String, timeout_ms: i64) -> Result<String, SkillError> {
    let resp = http_request(HttpRequestOptions {
        method: "HEAD".into(),
        url,
        timeout_ms,
        ..Default::default()
    })?;

    let hdrs: serde_json::Map<String, Value> = resp
        .headers
        .iter()
        .map(|h| (h.name.clone(), Value::String(h.value.clone())))
        .collect();
    let result = serde_json::json!({ "status": resp.status, "headers": hdrs });
    Ok(result.to_string())
}
