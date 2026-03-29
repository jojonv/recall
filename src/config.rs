use serde::Deserialize;
use std::path::PathBuf;
use std::fs;
use dirs::home_dir;

#[derive(Deserialize, Default, Debug, PartialEq)]
pub struct Config {
    pub file: Option<String>,
}

pub fn load_config(explicit_path: Option<PathBuf>) -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = if let Some(path) = explicit_path {
        path
    } else {
        let Some(mut path) = home_dir() else {
            return Ok(Config::default());
        };
        path.push(".recall");
        path.push("config.toml");
        path
    };

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(config_path)?;
    if content.trim().is_empty() {
        return Ok(Config::default());
    }
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

pub fn resolve_file_path(config: &Config) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(file_path) = &config.file {
        if let Some(stripped) = file_path.strip_prefix("~/") {
            if let Some(home) = home_dir() {
                return Ok(home.join(stripped));
            }
        } else if file_path == "~" {
            if let Some(home) = home_dir() {
                return Ok(home);
            }
        }
        return Ok(PathBuf::from(file_path));
    }

    let mut path = home_dir().ok_or("Could not find home directory")?;
    path.push(".recall");
    path.push("notes.md");
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_resolve_file_path_default() {
        let config = Config::default();
        let path = resolve_file_path(&config).unwrap();
        assert!(path.ends_with(".recall/notes.md") || path.ends_with(".recall\\notes.md"));
    }

    #[test]
    fn test_resolve_file_path_custom() {
        let config = Config {
            file: Some("/tmp/test_notes.md".to_string()),
        };
        let path = resolve_file_path(&config).unwrap();
        assert_eq!(path.to_str().unwrap(), "/tmp/test_notes.md");
    }

    #[test]
    fn test_resolve_file_path_tilde() {
        let config = Config {
            file: Some("~/vault/notes.md".to_string()),
        };
        let path = resolve_file_path(&config).unwrap();
        let home = home_dir().unwrap();
        let expected = home.join("vault/notes.md");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_resolve_file_path_tilde_only() {
        let config = Config {
            file: Some("~/".to_string()),
        };
        let path = resolve_file_path(&config).unwrap();
        let home = home_dir().unwrap();
        assert_eq!(path, home);
    }

    #[test]
    fn test_load_config_valid() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "file = '/path/to/notes.md'").unwrap();
        
        let config = load_config(Some(file.path().to_path_buf())).unwrap();
        assert_eq!(config.file, Some("/path/to/notes.md".to_string()));
    }

    #[test]
    fn test_load_config_missing_file() {
        let non_existent = PathBuf::from("does_not_exist_12345.toml");
        let config = load_config(Some(non_existent)).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_load_config_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let config = load_config(Some(file.path().to_path_buf())).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid = [toml").unwrap();
        
        let result = load_config(Some(file.path().to_path_buf()));
        assert!(result.is_err());
    }
}
