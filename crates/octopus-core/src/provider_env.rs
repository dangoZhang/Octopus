use std::env;
use std::fs;
use std::path::Path;

pub(crate) struct ProviderEnvGuard {
    saved: Vec<(String, Option<String>)>,
}

impl ProviderEnvGuard {
    pub(crate) fn auto(path: impl AsRef<Path>) -> Self {
        Self::from_path_missing_only(path).unwrap_or_default()
    }

    fn from_path_missing_only(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => {
                return Err(format!("cannot read {}: {error}", path.display()));
            }
        };
        let overlay = crate::app_bridge::parse_env_overlay(&content);
        let mut saved = Vec::new();
        for (key, value) in overlay {
            if env::var_os(&key).is_some() {
                continue;
            }
            saved.push((key.clone(), None));
            env::set_var(key, value);
        }
        Ok(Self { saved })
    }
}

impl Default for ProviderEnvGuard {
    fn default() -> Self {
        Self { saved: Vec::new() }
    }
}

impl Drop for ProviderEnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.saved.iter().rev() {
            if let Some(value) = value {
                env::set_var(key, value);
            } else {
                env::remove_var(key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProviderEnvGuard;
    use std::fs;

    #[test]
    fn provider_env_loads_missing_values_without_overriding_shell() {
        let dir =
            std::env::temp_dir().join(format!("octopus-provider-env-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("llm.env");
        fs::write(
            &path,
            "export OCTOPUS_ENV_TEST_MODEL=from-file\nexport OCTOPUS_ENV_TEST_API_KEY=from-file\n",
        )
        .unwrap();
        let old_model = std::env::var("OCTOPUS_ENV_TEST_MODEL").ok();
        let old_key = std::env::var("OCTOPUS_ENV_TEST_API_KEY").ok();
        std::env::set_var("OCTOPUS_ENV_TEST_MODEL", "from-shell");
        std::env::remove_var("OCTOPUS_ENV_TEST_API_KEY");

        {
            let _guard = ProviderEnvGuard::from_path_missing_only(&path).unwrap();
            assert_eq!(
                std::env::var("OCTOPUS_ENV_TEST_MODEL").unwrap(),
                "from-shell"
            );
            assert_eq!(
                std::env::var("OCTOPUS_ENV_TEST_API_KEY").unwrap(),
                "from-file"
            );
        }

        assert_eq!(
            std::env::var("OCTOPUS_ENV_TEST_MODEL").unwrap(),
            "from-shell"
        );
        assert!(std::env::var("OCTOPUS_ENV_TEST_API_KEY").is_err());
        restore_env("OCTOPUS_ENV_TEST_MODEL", old_model);
        restore_env("OCTOPUS_ENV_TEST_API_KEY", old_key);
        let _ = fs::remove_dir_all(&dir);
    }

    fn restore_env(key: &str, value: Option<String>) {
        if let Some(value) = value {
            std::env::set_var(key, value);
        } else {
            std::env::remove_var(key);
        }
    }
}
