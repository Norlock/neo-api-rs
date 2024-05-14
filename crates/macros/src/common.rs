pub fn sanitize_field_string(name: String) -> String {
    if let Some(suffix) = name.strip_prefix("r#") {
        suffix.to_string()
    } else {
        name
    }
}
