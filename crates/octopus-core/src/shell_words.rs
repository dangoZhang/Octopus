pub(crate) fn shell_command(command: &[String]) -> String {
    command
        .iter()
        .map(|part| shell_arg(part))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn shell_arg(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "/._-=".contains(ch))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}
