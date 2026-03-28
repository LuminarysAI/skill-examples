// Package main implements js-engine-skill with lmsk annotations.
// The developer writes only this file. Everything else is generated.
//
// @skill:id      ai.luminarys.go.js-engine
// @skill:name    "JS Engine Skill"
// @skill:version 1.0.0
// @skill:desc    "Sandboxed JavaScript execution: run ES5.1-compatible code, intercept console.log/info/warn/error, return aggregated output as string."
//
// Build:
//
//	go generate ./...
//	GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o js-engine-skill.wasm .
//
//go:generate lmsk -lang go -verbose .
package main

import (
	"strings"

	sdk "github.com/LuminarysAI/sdk-go"
	"github.com/dop251/goja"
)

// main is intentionally empty — init() in the generated skill_gen.go handles registration.

// ExecuteJS executes JavaScript code in an isolated goja runtime and captures console output.
// All console.log/info/warn/error calls are intercepted and returned as a single string.
//
// @skill:method execute "Execute JavaScript code and return captured console output."
// @skill:param content required "JavaScript source code to execute" example:"console.log('Hello'); 2 + 2;"
// @skill:result "Captured stdout from console.* calls as a single string (newline-separated)"
func ExecuteJS(ctx *sdk.Context, content string) (string, error) {
	vm := goja.New()

	var output strings.Builder

	console := vm.NewObject()
	_ = console.Set("log", func(call goja.FunctionCall) goja.Value {
		args := make([]string, len(call.Arguments))
		for i, arg := range call.Arguments {
			args[i] = arg.String()
		}
		// Append to output buffer.
		output.WriteString(strings.Join(args, " "))
		output.WriteByte('\n')
		return goja.Undefined()
	})

	// Redirect all log levels to the same handler.
	_ = console.Set("info", console.Get("log"))
	_ = console.Set("warn", console.Get("log"))
	_ = console.Set("error", console.Get("log"))

	// Expose console to the global JS scope.
	_ = vm.Set("console", console)

	// Execute the script.
	_, err := vm.RunString(content)
	if err != nil {
		return "", err
	}

	// Return captured output.
	result := strings.TrimSuffix(output.String(), "\n")
	return result, nil
}
