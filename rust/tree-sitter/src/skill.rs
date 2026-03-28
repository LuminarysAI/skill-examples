//! tree-sitter-skill — parse source code into AST/symbols using tree-sitter.
//!
//! All language grammars are compiled into the WASM binary. No external
//! dependencies at runtime.
//!
//! @skill:id      ai.luminarys.rust.tree-sitter
//! @skill:name    "Tree-sitter Parser"
//! @skill:version 1.0.0
//! @skill:desc    "Parse source code into AST and symbols. Supports Go, Python, JavaScript, TypeScript, Rust, C, Java, JSON, Bash, HTML, CSS."

use luminarys_sdk::prelude::*;
use serde_json::{json, Value};
use tree_sitter::{Language, Parser, Node};

// ── Language registry ───────────────────────────────────────────────────────

fn get_language(name: &str) -> Option<Language> {
    match name {
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "javascript" | "js" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "rust" | "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "c" | "h" => Some(tree_sitter_c::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "json" => Some(tree_sitter_json::LANGUAGE.into()),
        "bash" | "sh" => Some(tree_sitter_bash::LANGUAGE.into()),
        "typescript" | "ts" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "html" => Some(tree_sitter_html::LANGUAGE.into()),
        "css" => Some(tree_sitter_css::LANGUAGE.into()),
        _ => None,
    }
}

fn detect_language(filename: &str) -> Option<&'static str> {
    let ext = filename.rsplit('.').next()?;
    match ext {
        "go" => Some("go"),
        "py" => Some("python"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "rs" => Some("rust"),
        "c" | "h" => Some("c"),
        "java" => Some("java"),
        "json" => Some("json"),
        "sh" | "bash" => Some("bash"),
        "ts" => Some("typescript"),
        "tsx" => Some("tsx"),
        "html" | "htm" => Some("html"),
        "css" => Some("css"),
        "vue" => Some("html"),  // Vue SFC parsed as HTML
        _ => None,
    }
}

// ── Skill methods ───────────────────────────────────────────────────────────

/// Parse source code and return top-level symbols (functions, classes, etc.)
///
/// @skill:method parse "Parse source code and return symbols (functions, classes, structs, etc.)"
/// @skill:param  code     required "Source code to parse"
/// @skill:param  language optional "Language name (go, python, javascript, rust, c, java, json, bash). Auto-detect from filename if empty."
/// @skill:param  filename optional "Filename for language auto-detection (e.g. main.go)"
/// @skill:result "JSON with symbols: name, type, start/end line"
pub fn parse(
    _ctx: &mut Context,
    code: String,
    language: String,
    filename: String,
) -> Result<String, SkillError> {
    let lang_name = resolve_language(&language, &filename)?;
    let lang = get_language(lang_name)
        .ok_or_else(|| SkillError(format!("unsupported language: {}", lang_name)))?;

    let mut parser = Parser::new();
    parser.set_language(&lang)
        .map_err(|e| SkillError(format!("set language: {}", e)))?;

    let tree = parser.parse(&code, None)
        .ok_or_else(|| SkillError("parse failed".into()))?;

    let root = tree.root_node();
    let mut symbols = Vec::new();
    collect_symbols(&root, &code, &mut symbols);

    let result = json!({
        "language": lang_name,
        "filename": filename,
        "symbols": symbols,
    });

    Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
}

/// Parse a file from the filesystem and return top-level symbols.
/// Language is auto-detected from the file extension.
///
/// @skill:method parse_file "Read a file and parse it into symbols. Language is auto-detected from extension."
/// @skill:param  path     required "Absolute path to the source file"
/// @skill:param  language optional "Override language detection (e.g. go, python, rust)"
/// @skill:result "JSON with symbols: name, type, start/end line"
pub fn parse_file(
    _ctx: &mut Context,
    path: String,
    language: String,
) -> Result<String, SkillError> {
    let data = fs_read(&path)?;
    let code = String::from_utf8_lossy(&data).into_owned();

    let filename = path.rsplit('/').next()
        .or_else(|| path.rsplit('\\').next())
        .unwrap_or(&path);

    let lang_name = resolve_language(&language, filename)?;
    let lang = get_language(lang_name)
        .ok_or_else(|| SkillError(format!("unsupported language: {}", lang_name)))?;

    let mut parser = Parser::new();
    parser.set_language(&lang)
        .map_err(|e| SkillError(format!("set language: {}", e)))?;

    let tree = parser.parse(&code, None)
        .ok_or_else(|| SkillError("parse failed".into()))?;

    let root = tree.root_node();
    let mut symbols = Vec::new();
    collect_symbols(&root, &code, &mut symbols);

    let result = json!({
        "language": lang_name,
        "filename": filename,
        "symbols": symbols,
    });

    Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
}

/// Parse source code and return the full AST as JSON tree.
///
/// @skill:method ast "Parse source code and return the full syntax tree."
/// @skill:param  code     required "Source code to parse"
/// @skill:param  language optional "Language name. Auto-detect from filename if empty."
/// @skill:param  filename optional "Filename for language auto-detection"
/// @skill:param  max_depth optional "Maximum tree depth to return (0 = unlimited)"
/// @skill:result "JSON syntax tree"
pub fn ast(
    _ctx: &mut Context,
    code: String,
    language: String,
    filename: String,
    max_depth: i64,
) -> Result<String, SkillError> {
    let lang_name = resolve_language(&language, &filename)?;
    let lang = get_language(lang_name)
        .ok_or_else(|| SkillError(format!("unsupported language: {}", lang_name)))?;

    let mut parser = Parser::new();
    parser.set_language(&lang)
        .map_err(|e| SkillError(format!("set language: {}", e)))?;

    let tree = parser.parse(&code, None)
        .ok_or_else(|| SkillError("parse failed".into()))?;

    let depth_limit = if max_depth <= 0 { usize::MAX } else { max_depth as usize };
    let root_json = node_to_json(&tree.root_node(), &code, 0, depth_limit);

    let result = json!({
        "language": lang_name,
        "filename": filename,
        "root": root_json,
    });

    Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
}

/// List supported languages.
///
/// @skill:method languages "List supported languages and their file extensions."
/// @skill:result "JSON array of supported languages"
pub fn languages(_ctx: &mut Context) -> Result<String, SkillError> {
    let langs = json!([
        {"name": "go", "extensions": [".go"]},
        {"name": "python", "extensions": [".py"]},
        {"name": "javascript", "extensions": [".js", ".mjs", ".cjs"]},
        {"name": "rust", "extensions": [".rs"]},
        {"name": "c", "extensions": [".c", ".h"]},
        {"name": "java", "extensions": [".java"]},
        {"name": "json", "extensions": [".json"]},
        {"name": "bash", "extensions": [".sh", ".bash"]},
        {"name": "typescript", "extensions": [".ts"]},
        {"name": "tsx", "extensions": [".tsx"]},
        {"name": "html", "extensions": [".html", ".htm", ".vue"]},
        {"name": "css", "extensions": [".css"]},
    ]);
    Ok(serde_json::to_string_pretty(&langs).unwrap_or_default())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn resolve_language<'a>(language: &'a str, filename: &str) -> Result<&'a str, SkillError> {
    if !language.is_empty() {
        return Ok(language);
    }
    if !filename.is_empty() {
        if let Some(detected) = detect_language(filename) {
            // Leak is fine — these are static strings
            return Ok(detected);
        }
    }
    Err(SkillError("language not specified and cannot be auto-detected from filename".into()))
}

fn collect_symbols(node: &Node, source: &str, symbols: &mut Vec<Value>) {
    if !node.is_named() {
        return;
    }

    let kind = node.kind();

    // Top-level symbol types across all supported languages (deduplicated).
    let is_symbol = matches!(kind,
        // Go
        "function_declaration" | "method_declaration" | "type_declaration" |
        "type_spec" | "const_declaration" | "var_declaration" |
        // Python
        "function_definition" | "class_definition" | "decorated_definition" |
        // JavaScript / TypeScript (unique to JS/TS)
        "class_declaration" | "lexical_declaration" |
        "export_statement" | "variable_declaration" |
        // Rust
        "function_item" | "struct_item" | "enum_item" | "impl_item" |
        "trait_item" | "mod_item" | "const_item" | "static_item" |
        // C (unique to C)
        "declaration" | "struct_specifier" |
        "enum_specifier" | "type_definition" |
        // Java (unique to Java)
        "interface_declaration" |
        "constructor_declaration" | "field_declaration"
    );

    if is_symbol {
        let name = extract_name(node, source);
        symbols.push(json!({
            "type": kind,
            "name": name,
            "start_line": node.start_position().row + 1,
            "end_line": node.end_position().row + 1,
            "start_col": node.start_position().column,
            "end_col": node.end_position().column,
        }));
    }

    // Recurse into children (but not too deep for symbols).
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_symbols(&child, source, symbols);
    }
}

fn extract_name<'a>(node: &Node<'a>, source: &'a str) -> String {
    // Try to find a "name" or "identifier" child.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "identifier" || kind == "name" || kind == "type_identifier"
            || kind == "field_identifier"
        {
            return child.utf8_text(source.as_bytes()).unwrap_or("?").to_string();
        }
    }
    // Fallback: first named child.
    if let Some(first) = node.named_child(0) {
        if first.kind() == "identifier" || first.kind() == "name" {
            return first.utf8_text(source.as_bytes()).unwrap_or("?").to_string();
        }
    }
    "?".to_string()
}

fn node_to_json(node: &Node, source: &str, depth: usize, max_depth: usize) -> Value {
    let mut obj = json!({
        "type": node.kind(),
        "named": node.is_named(),
        "start": [node.start_position().row + 1, node.start_position().column],
        "end": [node.end_position().row + 1, node.end_position().column],
    });

    if node.child_count() == 0 {
        // Leaf node — include text.
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            if text.len() <= 200 {
                obj["text"] = json!(text);
            }
        }
    } else if depth < max_depth {
        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            children.push(node_to_json(&child, source, depth + 1, max_depth));
        }
        obj["children"] = json!(children);
    }

    obj
}
