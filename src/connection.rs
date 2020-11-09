use async_std::{
  net::TcpStream,
  prelude::*,
};
use async_tls::TlsAcceptor;
use log;

use crate::config::ServerConfig;
use crate::request;
use crate::Result;
use crate::status::Status;

pub async fn handle_connection(acceptor: &TlsAcceptor, tcp_stream: &mut TcpStream, config: &ServerConfig) -> Result<()> {
  let peer_addr = tcp_stream.peer_addr()?;
  let handshake = acceptor.accept(tcp_stream);
  let mut tls_stream = handshake.await?;

  match request::parse(&mut tls_stream).await {
    Ok(url) => {
      let hostname = url.host_str().ok_or(format!("Request from {} containing invalid URL: {}", peer_addr, url))?; 
      if config.server_name.iter().any(|name| name.as_str() == hostname) {
        match request::handle(&mut tls_stream, &url, config).await {
          Ok(status) => log::info!("Handled request {} from {} with status {}", url, peer_addr, status),
          Err(err)   => log::warn!("Error handling request {} from {}: {}", url, peer_addr, err),
        }
      }
      else {
        log::warn!("Received request to hostname {} for which no server configuration exists", hostname);
      }
    },
    Err(err) => {
      tls_stream.write(Status::BadRequest.to_bytes()).await?;
      log::warn!("Bad request from {}: {}", peer_addr, err);
    },
  }

  tls_stream.flush().await?;
  Ok(())
}
