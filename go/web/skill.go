// Package main implements web-skill — outbound HTTP/HTTPS requests.
//
// @skill:id      ai.luminarys.go.web
// @skill:name    "Web Skill"
// @skill:version 1.0.0
// @skill:desc    "Outbound HTTP and HTTPS requests: GET, POST, and custom requests with headers."
//
//go:generate lmsk -lang go -verbose .
package main

import (
	"encoding/json"
	"fmt"

	sdk "github.com/LuminarysAI/sdk-go"
)

// Get performs an HTTP GET request and returns the response body as text.
// @skill:method get "Perform an HTTP GET request and return the response body."
// @skill:param  url        required "Full URL to fetch (must match allowlist)" example:https://api.example.com/data
// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30 s default" default:0
// @skill:result "Response body as UTF-8 text"
func Get(ctx *sdk.Context, url string, timeoutMs int64) (string, error) {
	resp, err := sdk.HttpGet(url, timeoutMs, 0)
	if err != nil {
		return "", err
	}
	if resp.Status >= 400 {
		return "", fmt.Errorf("HTTP %d: %s", resp.Status, string(resp.Body))
	}
	return string(resp.Body), nil
}

// GetJSON performs an HTTP GET and parses the response as JSON, returning it
// as a pretty-printed string suitable for the LLM to read.
// @skill:method get_json "Fetch a URL and return the parsed JSON response."
// @skill:param  url        required "Full URL to fetch (must return JSON)"
// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30 s default" default:0
// @skill:result "Pretty-printed JSON response"
func GetJSON(ctx *sdk.Context, url string, timeoutMs int64) (string, error) {
	resp, err := sdk.HttpGet(url, timeoutMs, 0)
	if err != nil {
		return "", err
	}
	if resp.Status >= 400 {
		return "", fmt.Errorf("HTTP %d", resp.Status)
	}
	// Re-indent for readability
	var v interface{}
	if err := json.Unmarshal(resp.Body, &v); err != nil {
		// Not valid JSON — return raw body
		return string(resp.Body), nil
	}
	pretty, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		return string(resp.Body), nil
	}
	return string(pretty), nil
}

// Post performs an HTTP POST with a JSON body.
// @skill:method post "Perform an HTTP POST request with a JSON body."
// @skill:param  url        required "Target URL (must match allowlist)"
// @skill:param  body       required "JSON string to send as the request body"
// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30 s default" default:0
// @skill:result "Response body as text"
func Post(ctx *sdk.Context, url string, body string, timeoutMs int64) (string, error) {
	resp, err := sdk.HttpPost(url, []byte(body), "application/json", timeoutMs, 0)
	if err != nil {
		return "", err
	}
	if resp.Status >= 400 {
		return "", fmt.Errorf("HTTP %d: %s", resp.Status, string(resp.Body))
	}
	return string(resp.Body), nil
}

// Request performs a fully customised HTTP request with explicit headers.
// @skill:method request "Perform a custom HTTP request with explicit method and headers."
// @skill:param  method     required "HTTP method: GET, POST, PUT, PATCH, DELETE" example:POST
// @skill:param  url        required "Target URL (must match allowlist)"
// @skill:param  body       optional "Request body as text or JSON string" default:""
// @skill:param  headers    optional "Headers as JSON-encoded string, e.g. {\"Content-Type\":\"application/json\"}" default:""
// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30 s default" default:0
// @skill:result "Response with status and body"
func Request(ctx *sdk.Context, method, url, body, headersJSON string, timeoutMs int64) (string, error) {
	hdrs := sdk.HeadersFromJSON(headersJSON)

	resp, err := sdk.HttpRequest(sdk.HttpRequestOptions{
		Method:          method,
		URL:             url,
		Headers:         hdrs,
		Body:            []byte(body),
		TimeoutMs:       timeoutMs,
		FollowRedirects: true,
	})
	if err != nil {
		return "", err
	}

	result := map[string]interface{}{
		"status": resp.Status,
		"body":   string(resp.Body),
	}
	b, _ := json.Marshal(result)
	return string(b), nil
}

// Head performs an HTTP HEAD request and returns status and headers.
// @skill:method head "Perform an HTTP HEAD request and return status and headers."
// @skill:param  url        required "Target URL (must match allowlist)"
// @skill:param  timeout_ms optional "Request timeout in milliseconds. 0 = 30 s default" default:0
// @skill:result "JSON with status code and response headers"
func Head(ctx *sdk.Context, url string, timeoutMs int64) (string, error) {
	resp, err := sdk.HttpRequest(sdk.HttpRequestOptions{
		Method:    "HEAD",
		URL:       url,
		TimeoutMs: timeoutMs,
	})
	if err != nil {
		return "", err
	}

	hdrs := make(map[string]string, len(resp.Headers))
	for _, h := range resp.Headers {
		hdrs[h.Name] = h.Value
	}
	result := map[string]interface{}{
		"status":  resp.Status,
		"headers": hdrs,
	}
	b, _ := json.Marshal(result)
	return string(b), nil
}
