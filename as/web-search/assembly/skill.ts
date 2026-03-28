/**
 * @skill:id      ai.luminarys.as.web-search
 * @skill:name    "Web Search (AS)"
 * @skill:version 1.0.0
 * @skill:desc    "Search the web and return results as JSON."
 * @skill:sdk     "@luminarys/sdk-as"
 * @skill:require http **
 */

import { Context,  httpGet } from "@luminarys/sdk-as";

// @skill:method search "Search the web and return JSON results."
// @skill:param  query required "Search query"
// @skill:result "JSON search results"
export function search(_ctx: Context, query: string): string {
  // Use DuckDuckGo Instant Answer API (no auth required).
  const encoded = encodeURIComponent(query);
  const url = "https://api.duckduckgo.com/?q=" + encoded + "&format=json&no_html=1";
  const resp = httpGet(url, 10000, 0);
  return String.UTF8.decode(resp.body.buffer);
}

/** Minimal URL encoding for query strings. */
function encodeURIComponent(s: string): string {
  let result = "";
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if ((c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57) ||
        c == 45 || c == 95 || c == 46 || c == 126) {
      result += String.fromCharCode(c);
    } else if (c == 32) {
      result += "+";
    } else {
      const hex = c.toString(16).toUpperCase();
      result += "%" + (hex.length < 2 ? "0" + hex : hex);
    }
  }
  return result;
}
