use std::{
  error::Error,
  fs,
  path::PathBuf,
  //sync::Arc,
};
use serde::{Deserialize, Serialize};
//use serde_json::Result;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
  pub listen_addr: String,
  pub tls_certificate: String,
  pub tls_certificate_key: String,

  pub index: Vec<String>,
  pub server_root: String,
}

pub fn load(path: PathBuf) -> Result<ServerConfig> {
  let data = fs::read_to_string(path)?;

  let c: ServerConfig = serde_json::from_str(&data)?;

  println!("test_parse: {:?}", c);

  Ok(c)
}