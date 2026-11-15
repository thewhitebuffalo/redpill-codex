//! redpill: local Responses servers (llama.cpp) skip `namespace` tool groups, so for
//! non-OpenAI providers namespaced tools are sent as flat top-level functions and
//! calls to those flat names are mapped back to their namespace.

use codex_protocol::DEFAULT_FUNCTION_NAMESPACE;
use serde_json::Value;

/// Flat function name for a namespaced tool (`mcp__server__` + `tool`).
pub(crate) fn flat_name(namespace: &str, name: &str) -> String {
    if namespace.is_empty() || namespace == DEFAULT_FUNCTION_NAMESPACE {
        name.to_string()
    } else if namespace.ends_with("__") {
        format!("{namespace}{name}")
    } else {
        format!("{namespace}__{name}")
    }
}

/// Lift function tools out of `namespace` groups; drop tool types local servers cannot run.
pub(crate) fn flatten_tools(tools: Value) -> Value {
    let Value::Array(items) = tools else {
        return tools;
    };
    let mut out = Vec::with_capacity(items.len());
    for tool in items {
        match tool.get("type").and_then(Value::as_str) {
            Some("namespace") => {
                let namespace = tool.get("name").and_then(Value::as_str).unwrap_or_default();
                if let Some(Value::Array(nested)) = tool.get("tools") {
                    for nested_tool in nested {
                        if nested_tool.get("type").and_then(Value::as_str) != Some("function") {
                            continue;
                        }
                        let mut flat = nested_tool.clone();
                        if let Some(name) = nested_tool.get("name").and_then(Value::as_str) {
                            flat["name"] = Value::String(flat_name(namespace, name));
                        }
                        out.push(flat);
                    }
                }
            }
            _ => out.push(tool),
        }
    }
    Value::Array(out)
}
