use dirs::home_dir;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Default, Debug, PartialEq)]
pub struct Config {
    pub file: Option<String>,
    pub notebooks: Option<HashMap<String, String>>,
}

impl Config {
    pub fn resolve_notebook_path(&self, name: &str) -> Option<PathBuf> {
        let notebooks = self.notebooks.as_ref()?;
        let path = notebooks.get(name)?;
        Self::expand_path(path)
    }

    pub fn expand_path(path: &str) -> Option<PathBuf> {
        if let Some(stripped) = path.strip_prefix("~/") {
            home_dir().map(|home| home.join(stripped))
        } else if path == "~" {
            home_dir()
        } else {
            Some(PathBuf::from(path))
        }
    }
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
    match toml::from_str::<Config>(&content) {
        Ok(config) => Ok(config),
        Err(e) => {
            // Check if the error might be due to Windows paths with backslashes
            // in double-quoted strings (TOML requires escaping backslashes in double quotes)
            if content.contains('"') {
                // Look for double-quoted strings containing backslashes
                let scan_result = content.chars().fold(
                    (false, false),
                    |(in_double_quote, found_backslash), c| {
                        if c == '"' {
                            (!in_double_quote, found_backslash)
                        } else if in_double_quote && c == '\\' {
                            (true, true)
                        } else {
                            (in_double_quote, found_backslash)
                        }
                    },
                );

                if scan_result.1 {
                    // found_backslash is true
                    return Err(format!(
                        "Failed to parse config.toml: {}\n\nHint: Windows paths with backslashes must use single quotes:\n\n  [notebooks]\n  w = 'C:\\Data\\Obsidian\\vault\\notes.md'\n\nOr escape the backslashes in double quotes:\n\n  w = \"C:\\\\Data\\\\Obsidian\\\\vault\\\\notes.md\"",
                        e
                    ).into());
                }
            }
            Err(e.into())
        }
    }
}

pub fn resolve_file_path(config: &Config) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(file_path) = &config.file {
        if let Some(path) = Config::expand_path(file_path) {
            return Ok(path);
        }
        return Err("Could not resolve file path: home directory not found".into());
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
            notebooks: None,
        };
        let path = resolve_file_path(&config).unwrap();
        assert_eq!(path.to_str().unwrap(), "/tmp/test_notes.md");
    }

    #[test]
    fn test_resolve_file_path_tilde() {
        let config = Config {
            file: Some("~/vault/notes.md".to_string()),
            notebooks: None,
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
            notebooks: None,
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

    #[test]
    fn test_resolve_notebook_path() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "file = '/path/to/default.md'").unwrap();
        writeln!(file, "[notebooks]").unwrap();
        writeln!(file, "w = '/path/to/work.md'").unwrap();
        writeln!(file, "p = '~/notes/personal.md'").unwrap();

        let config = load_config(Some(file.path().to_path_buf())).unwrap();

        // Test work notebook
        let work_path = config.resolve_notebook_path("w");
        assert!(work_path.is_some());
        assert_eq!(work_path.unwrap().to_str().unwrap(), "/path/to/work.md");

        // Test personal notebook with tilde expansion
        let personal_path = config.resolve_notebook_path("p");
        assert!(personal_path.is_some());
        let home = home_dir().unwrap();
        assert_eq!(personal_path.unwrap(), home.join("notes/personal.md"));

        // Test non-existent notebook
        assert!(config.resolve_notebook_path("x").is_none());
    }

    #[test]
    fn test_resolve_notebook_path_no_notebooks() {
        let config = Config::default();
        assert!(config.resolve_notebook_path("w").is_none());
    }

    #[test]
    fn test_load_config_windows_path_hint() {
        let mut file = NamedTempFile::new().unwrap();
        // Write raw TOML with unescaped backslashes in double quotes (invalid TOML)
        // Using r#"..."# style to write literal content
        file.write_all(
            br#"file = 'C:\some\path.md'

[notebooks]
w = "C:\Data\notes.md"
"#,
        )
        .unwrap();

        let result = load_config(Some(file.path().to_path_buf()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to parse config.toml"));
        assert!(err.contains("Hint: Windows paths with backslashes must use single quotes"));
    }

    #[test]
    fn test_load_config_other_errors_no_windows_hint() {
        let mut file = NamedTempFile::new().unwrap();
        // Write invalid TOML that doesn't involve backslashes (missing closing bracket)
        file.write_all(
            br#"file = '/path/to/notes.md'

[notebooks
w = '/path/to/work.md'
"#,
        )
        .unwrap();

        let result = load_config(Some(file.path().to_path_buf()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // Just verify it's a TOML error without the Windows hint
        assert!(!err.contains("Hint: Windows paths with backslashes must use single quotes"));
    }

    #[test]
    fn test_load_config_properly_escaped_backslashes() {
        let mut file = NamedTempFile::new().unwrap();
        // Write valid TOML with properly escaped backslashes in double quotes
        file.write_all(
            br#"file = 'C:\some\path.md'

[notebooks]
w = "C:\\Data\\notes.md"
"#,
        )
        .unwrap();

        let result = load_config(Some(file.path().to_path_buf()));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.file, Some("C:\\some\\path.md".to_string()));
        assert_eq!(
            config.resolve_notebook_path("w"),
            Some(PathBuf::from("C:\\Data\\notes.md"))
        );
    }
}
