use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{Result, eyre::eyre};

pub fn try_get_path(value: &str, working_dir: Option<String>) -> Result<Option<Utf8PathBuf>> {
    if !looks_like_path(value) {
        return Ok(None);
    }

    let value_expanded = expand_home(value)?;

    let working_dir = if let Some(working_dir) = working_dir {
        working_dir
    } else {
        Utf8PathBuf::from_path_buf(std::env::current_dir()?)
            .map_err(|_| eyre!("unable to process non UTF-8 path"))?
            .into_string()
    };
    let base = Utf8Path::new(&working_dir);
    let raw = Utf8Path::new(&value_expanded);

    let joined: Utf8PathBuf = if raw.is_absolute() || value_expanded.starts_with(r"\\") {
        raw.to_path_buf()
    } else {
        base.join(raw)
    };

    let canon_path = dunce::canonicalize(joined)
        .map_err(|err| eyre!("unable to process path: '{value}'\n{err}"))?;

    let path = Utf8PathBuf::from_path_buf(canon_path)
        .map_err(|_| eyre!("unable to process non UTF-8 path: {value:?}"))?;

    Ok(Some(path))
}

fn looks_like_path(s: &str) -> bool {
    // absolute (Unix) ("/...")
    if s.starts_with('/') {
        return true;
    }

    // relative markers ("./", "../", ".\", "..\")
    if s.starts_with("./") || s.starts_with(".\\") || s.starts_with("../") || s.starts_with("..\\")
    {
        return true;
    }

    // home-ish ("~/", "~\")
    if s.starts_with("~/") || s.starts_with("~\\") {
        return true;
    }

    // contains a path separator ('/', '\')
    if s.contains('/') || s.contains('\\') {
        return true;
    }

    // UNC (Windows) ("\\server\share")
    if s.starts_with(r"\\") {
        return true;
    }

    // absolute (Windows) ("C:\...", "C:/...")
    let b = s.as_bytes();
    if b.len() >= 3 && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/') && b[0].is_ascii_alphabetic()
    {
        return true;
    }

    false
}

fn expand_home(value: &str) -> Result<String> {
    if let Some(rest) = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
    {
        let home =
            home::home_dir().ok_or_else(|| eyre!("unabl to resolve home directory for '~'"))?;
        let mut buf = Utf8PathBuf::from_path_buf(home)
            .map_err(|_| eyre!("home directory is not valid UTF-8"))?;
        buf.push(rest);
        Ok(buf.into_string())
    } else {
        Ok(value.to_string())
    }
}
