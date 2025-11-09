use toml::{Value};

pub fn ensure_config() {
  let config_dir = crate::APP_DIRS.config_dir.clone();
  if !config_dir.exists() {
    std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
  }

  let config_file = config_dir.join("config.toml");
  if !config_file.exists() {
    std::fs::write(&config_file, b"# Color Gen Configuration File\n").expect("Failed to create config file");
  }
}

pub fn get_config() -> Value {
  let config_path = crate::APP_DIRS.config_dir.join("config.toml");
  let config_str = std::fs::read_to_string(&config_path).expect("Failed to read config file");
  toml::from_str(&config_str).expect("Failed to parse config file")
}

pub fn save_config(config: &Value) {
  let config_path = crate::APP_DIRS.config_dir.join("config.toml");
  let toml_str = toml::to_string(config).expect("Failed to serialize config");
  std::fs::write(&config_path, toml_str).expect("Failed to write config file");
}