use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{Result, eyre::eyre};

pub fn try_get_path(value: &str) -> Result<Option<String>> {
    // Cross-platform “pathy” heuristics:
    // 1) absolute (Unix)   -> "/..."
    // 2) absolute (Windows)-> "C:\..." or "C:/..."
    // 3) UNC (Windows)     -> "\\server\share"
    // 4) relative markers  -> "./", "../", ".\", "..\"
    // 5) home-ish          -> "~/" or "~\"
    // 6) contains a path separator ('/' or '\')

    let path = Utf8Path::new(value);
    let mut is_path = false;

    if value.starts_with('/') {
        is_path = true;
    }
    if value.starts_with("./")
        || value.starts_with(".\\")
        || value.starts_with("../")
        || value.starts_with("..\\")
    {
        is_path = true;
    }
    if value.starts_with("~/") || value.starts_with("~\\") {
        is_path = true;
    }
    if value.contains('/') || value.contains('\\') {
        is_path = true;
    }
    // Windows drive letter "C:\..." or "C:/..."
    let bytes = value.as_bytes();
    if bytes.len() >= 3
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
        && bytes[0].is_ascii_alphabetic()
    {
        is_path = true;
    }
    // UNC path
    if value.starts_with(r"\\") {
        is_path = true;
    }

    Ok(if is_path {
        Some(
            Utf8PathBuf::from_path_buf(dunce::canonicalize(path)?)
                .map_err(|_| eyre!("unable to process non-utf8 path: {:?}", value))?
                .into_string(),
        )
    } else {
        None
    })
}
