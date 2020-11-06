use async_std::{
  net::TcpStream,
  prelude::*,
};
use async_tls::TlsAcceptor;
use std::{
  error::Error,
};
use url::Url;

use crate::config::ServerConfig;
use crate::request;
use crate::status::Status;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub async fn handle_connection(acceptor: &TlsAcceptor, tcp_stream: &mut TcpStream, config: &ServerConfig) -> Result<()> {
  let handshake = acceptor.accept(tcp_stream);
  let mut tls_stream = handshake.await?;

  if let Ok(url) = request::parse(&mut tls_stream).await {
    if let Some(hostname) = url.host_str() {
      if config.server_name.iter().any(|name| name.as_str() == hostname) {
        request::handle(&mut tls_stream, &url, config).await?;
      }
    }
  }
  else {
    tls_stream.write(Status::BadRequest.to_bytes()).await?;
  }

  tls_stream.flush().await?;
  Ok(())
}
