/**
 * @skill:id      ai.luminarys.as.web
 * @skill:name    "Web Skill (AS)"
 * @skill:version 1.0.0
 * @skill:desc    "HTTP client: GET, POST, HEAD, custom requests with headers."
 * @skill:sdk     "@luminarys/sdk-as"
 * @skill:require http **
 */

import { Context, httpGet, httpPost, httpRequest, headersFromJSON } from "@luminarys/sdk-as";

function bodyToString(body: Uint8Array): string {
  return String.UTF8.decode(body.buffer);
}

// @skill:method get "Perform an HTTP GET request."
// @skill:param  url required "Request URL"
// @skill:result "Response body as text"
export function get(_ctx: Context, url: string): string {
  const resp = httpGet(url, 0, 0);
  return bodyToString(resp.body);
}

// @skill:method get_json "Fetch a URL and return parsed JSON."
// @skill:param  url required "Request URL"
// @skill:result "JSON response"
export function getJson(_ctx: Context, url: string): string {
  const resp = httpGet(url, 0, 0);
  return bodyToString(resp.body);
}

// @skill:method post "Perform an HTTP POST request with JSON body."
// @skill:param  url  required "Request URL"
// @skill:param  body required "JSON request body"
// @skill:result "Response body as text"
export function post(_ctx: Context, url: string, body: string): string {
  const bodyBytes = Uint8Array.wrap(String.UTF8.encode(body));
  const resp = httpPost(url, bodyBytes, "application/json", 0);
  return bodyToString(resp.body);
}

// @skill:method head "Perform an HTTP HEAD request (headers only)."
// @skill:param  url required "Request URL"
// @skill:result "Response status"
export function head(_ctx: Context, url: string): string {
  const resp = httpRequest("HEAD", url);
  return "status: " + resp.status.toString();
}

// @skill:method request "Perform a custom HTTP request with method and headers."
// @skill:param  method  required "HTTP method (GET, POST, PUT, DELETE, etc.)"
// @skill:param  url     required "Request URL"
// @skill:param  body    optional "Request body"
// @skill:param  headers optional "Headers as JSON-encoded string, e.g. {\"Content-Type\":\"application/json\"}" default:""
// @skill:result "Response body as text"
export function request(_ctx: Context, method: string, url: string, body: string, headers: string): string {
  const bodyBytes = body.length > 0 ? Uint8Array.wrap(String.UTF8.encode(body)) : new Uint8Array(0);
  const hdrs = headersFromJSON(headers);
  const resp = httpRequest(method, url, bodyBytes, hdrs);
  return bodyToString(resp.body);
}
