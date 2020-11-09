use regex::Regex;
use std::{
  fs,
  path::PathBuf,
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
pub struct ServerConfig {
  pub listen_addr: String,
  pub tls_certificate: String,
  pub tls_certificate_key: String,

  pub server_name: Vec<String>,
  pub server_root: String,
  pub index: Vec<String>,
  #[serde(default, with = "serde_regex")]
  pub autoindex_rules: Vec<Regex>,
  #[serde(default)]
  pub rewrite_rules: Vec<RewriteRule>,
  #[serde(default)]
  pub redirect_rules: Vec<RewriteRule>,
}

pub fn load(path: PathBuf) -> Result<ServerConfig> {
  let data = fs::read_to_string(path)?;
  let c: ServerConfig = serde_json::from_str(&data)?;

  Ok(c)
}