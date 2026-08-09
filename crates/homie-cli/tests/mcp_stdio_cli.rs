use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn mcp_stdio_without_runtime_only_exposes_runtime_independent_tools() {
    let binary = env!("CARGO_BIN_EXE_homie");

    let tools = mcp_roundtrip(
        binary,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#,
    );
    assert_eq!(tools["id"], 1);
    let tool_names = tools["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect::<Vec<_>>();
    assert_eq!(tool_names, vec!["whoami"]);

    let list_agents = mcp_roundtrip(
        binary,
        r#"{"jsonrpc":"2.0","id":"a","method":"tools/call","params":{"name":"list_agents","arguments":{}}}"#,
    );
    assert_eq!(list_agents["id"], "a");
    assert_eq!(list_agents["error"]["code"], -32601);

    let whoami = mcp_roundtrip(
        binary,
        r#"{"jsonrpc":"2.0","id":"b","method":"tools/call","params":{"name":"whoami","arguments":{}}}"#,
    );
    assert_eq!(whoami["result"]["content"][0]["type"], "text");
    assert!(
        whoami["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("sessionId")
    );

    let unknown = mcp_roundtrip(
        binary,
        r#"{"jsonrpc":"2.0","id":"c","method":"tools/call","params":{"name":"missing","arguments":{}}}"#,
    );
    assert_eq!(unknown["error"]["code"], -32601);
}

fn mcp_roundtrip(binary: &str, line: &str) -> serde_json::Value {
    let mut child = Command::new(binary)
        .arg("mcp-stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn homie mcp-stdio");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        stdin.write_all(line.as_bytes()).expect("write line");
        stdin.write_all(b"\n").expect("write newline");
    }
    let output = child.wait_with_output().expect("wait output");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    serde_json::from_str(stdout.lines().next().expect("first line")).expect("json response")
}
