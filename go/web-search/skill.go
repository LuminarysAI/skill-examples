// Package main implements web-search-skill — search via Tavily API.
// Every SDK must pass this test before being considered compatible with the host.
//
// @skill:id      ai.luminarys.go.web-search
// @skill:name    "Web Search"
// @skill:version 1.0.0
// @skill:desc    "Performs web search. Returns structured JSON results with advanced depth."
//
//go:generate lmsk -lang go -verbose .
package main

import (
	"encoding/json"
	"errors"

	sdk "github.com/LuminarysAI/sdk-go"
)

// Search executes a web search query via Tavily API.
// @skill:method search "Search the web and return JSON results."
// @skill:param  query required "Search query string"
// @skill:result "JSON string containing search results"
func Search(ctx *sdk.Context, query string) (string, error) {
	apiKey := sdk.GetEnv("TAVILY_API_KEY")
	if apiKey == "" {
		return "", errors.New("TAVILY_API_KEY environment variable not set")
	}

	reqJson := struct {
		Query          string `json:"query"`
		SearchDepth    string `json:"search_depth"`
		IncludeFavicon bool   `json:"include_favicon"`
	}{
		Query:          query,
		SearchDepth:    "advanced",
		IncludeFavicon: false,
	}

	reqBody, _ := json.Marshal(reqJson)

	resp, err := sdk.HttpRequest(sdk.HttpRequestOptions{
		Method: "POST",
		URL:    "https://api.tavily.com/search",
		Headers: []sdk.Header{
			{Name: "Authorization", Value: "Bearer " + apiKey},
			{Name: "Content-Type", Value: "application/json"},
		},
		Body:            reqBody,
		TimeoutMs:       0,
		MaxBytes:        0,
		FollowRedirects: false,
		UseJar:          false,
	})

	return string(resp.Body), err
}
