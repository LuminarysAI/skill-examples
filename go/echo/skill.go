// Package main implements echo-skill — ABI compatibility smoke test.
// Every SDK must pass this test before being considered compatible with the host.
//
// @skill:id      ai.luminarys.go.echo
// @skill:name    "Echo Skill"
// @skill:version 1.0.0
// @skill:desc    "ABI compatibility smoke-test. Echoes payload, counts calls, pings."
//
//go:generate lmsk -lang go -verbose .
package main

import sdk "github.com/LuminarysAI/sdk-go"

// Echo returns the input string unchanged.
// @skill:method echo "Return the input string unchanged."
// @skill:param  message required "Any string"
// @skill:result "The same string"
func Echo(ctx *sdk.Context, message string) (string, error) {
	return message, nil
}

// Ping returns "pong".
// @skill:method ping "Health-check. Always returns pong."
// @skill:result "Always pong"
func Ping(ctx *sdk.Context) (string, error) {
	return "pong", nil
}

// Reverse returns the input string reversed.
// @skill:method reverse "Reverse the characters of a string."
// @skill:param  message required "String to reverse"
// @skill:result "Reversed string"
func Reverse(ctx *sdk.Context, message string) (string, error) {
	runes := []rune(message)
	for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
		runes[i], runes[j] = runes[j], runes[i]
	}
	return string(runes), nil
}
