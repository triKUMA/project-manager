pub fn get_base_key(key: &str, allow_implicit_scope: bool) -> &str {
    let key = if allow_implicit_scope {
        key.strip_prefix('!').unwrap_or(key)
    } else {
        key
    };

    key.split('?').next().unwrap()
}
