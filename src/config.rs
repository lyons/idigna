use regex::Regex;
use std::{
  collections::HashMap,
  fs,
  path::PathBuf,
  sync::Arc,
};
use serde::{Deserialize, Serialize};
use serde_regex;

use crate::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct RewriteRule {
  #[serde(with = "serde_regex")]
  pub pattern: Regex,
  pub substitution: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
  settings: BaseConfig,
  servers: Vec<ServerConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BaseConfig {
  pub listen_addr: String,
  pub tls_certificate: String,
  pub tls_certificate_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
  server_name: Vec<String>,
  pub server_root: String,
  pub index: Vec<String>,
  #[serde(default, with = "serde_regex")]
  pub autoindex_rules: Vec<Regex>,
  #[serde(default)]
  pub rewrite_rules: Vec<RewriteRule>,
  #[serde(default)]
  pub redirect_rules: Vec<RewriteRule>,
}

pub fn load(path: PathBuf) -> Result<(BaseConfig, HashMap<String, Arc<ServerConfig>>)> {
  let data = fs::read_to_string(path)?;
  let c: Config = serde_json::from_str(&data)?;

  let mut server_hash = HashMap::new();
  for server in c.servers {
    let hostnames = server.server_name.clone();
    let block = Arc::new(server);
    for name in hostnames {
      server_hash.insert(name, block.clone());
    }
  }

  Ok((c.settings, server_hash))
}