use rustls::{
  internal::pemfile::{
    certs,
    pkcs8_private_keys,
  },
  Certificate,
  NoClientAuth,
  PrivateKey,
  ServerConfig as TlsServerConfig,
};
use std::{
  fs::File,
  io::{self, BufReader},
  path::PathBuf,
};

use crate::config::ServerConfig;
use crate::Result;

fn load_certs(path: PathBuf) -> io::Result<Vec<Certificate>> {
  certs(&mut BufReader::new(File::open(path)?))
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid certificate"))
}

fn load_keys(path: PathBuf) -> io::Result<Vec<PrivateKey>> {
  pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid key"))
}

pub fn load_tls_config(options: &ServerConfig) -> Result<TlsServerConfig> {
  let certs = load_certs(PathBuf::from(&options.tls_certificate))?;
  let mut keys = load_keys(PathBuf::from(&options.tls_certificate_key))?;

  let mut config = TlsServerConfig::new(NoClientAuth::new());
  config
    .set_single_cert(certs, keys.remove(0))
    .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

  Ok(config)
}