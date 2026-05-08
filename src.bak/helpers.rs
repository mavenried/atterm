pub fn is_terminal_response(line: &str) -> bool {
    matches!(
        line.trim(),
        "OK" | "ERROR" | "NO CARRIER" | "NO ANSWER" | "NO DIALTONE" | "BUSY"
    ) || line.trim().starts_with("+CME ERROR")
        || line.trim().starts_with("+CMS ERROR")
}
