/// Returns true if the line is a final response that terminates an AT command exchange.
pub fn is_terminal_response(line: &str) -> bool {
    matches!(
        line.trim(),
        "OK" | "ERROR" | "NO CARRIER" | "NO ANSWER" | "NO DIALTONE" | "BUSY"
    ) || line.trim().starts_with("+CME ERROR")
        || line.trim().starts_with("+CMS ERROR")
}

/// Append \r\n framing expected by AT command modems.
pub fn frame_command(cmd: &str) -> String {
    format!("{}\r\n", cmd)
}
