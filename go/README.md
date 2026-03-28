# Go Skills

WASM skills built with the [Luminarys Go SDK](https://github.com/LuminarysAI/sdk-go).

## Prerequisites

- Go 1.23+
- `lmsk` CLI (skill toolchain)
- Signing key: `lmsk genkey`

## Build

```bash
cd echo
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o echo.wasm .
lmsk sign echo.wasm
```

**Windows (cmd.exe):**
```cmd
set GOOS=wasip1
set GOARCH=wasm
go build -buildmode=c-shared -o echo.wasm .
```

> **Important:** `-buildmode=c-shared` is required. Without it the binary exports `_start` instead of `_initialize` and will fail at runtime.

## Skills

| Skill | Description |
|-------|-------------|
| [echo](echo/) | Echo, ping, reverse — ABI smoke test |
| [fs](fs/) | File system operations (read, write, edit, search) |
| [web](web/) | HTTP/HTTPS outbound requests |
| [web-search](web-search/) | Web search via Tavily API |
| [git](git/) | Git repository operations |
| [archive](archive/) | Create and extract archives |
| [file-transfer](file-transfer/) | Copy files between cluster nodes |
| [go](go/) | Go toolchain and process management |
| [js-engine](js-engine/) | Sandboxed JavaScript execution |
