use async_std::{
  fs::File,
  io::{Read, Write},
  net::{
    TcpStream,
  },
  path::PathBuf,
  prelude::*,
};
use async_tls::TlsAcceptor;
// use rustls::{
//   internal::pemfile::{
//     certs,
//     pkcs8_private_keys,
//   },
//   Certificate,
//   NoClientAuth,
//   PrivateKey,
//   ServerConfig,
// };
use std::{
  error::Error,
  marker::Unpin,
};
use url::Url;

use super::status::Status;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub async fn handle_connection(acceptor: &TlsAcceptor, tcp_stream: &mut TcpStream) -> Result<()> {
  let peer_addr = tcp_stream.peer_addr()?;
  let local_addr = tcp_stream.local_addr()?;
  println!("Connection from {}", peer_addr);
  println!("Connection to {}", local_addr);

  let handshake = acceptor.accept(tcp_stream);
  let mut tls_stream = handshake.await?;

  if let Ok(url) = parse_request(&mut tls_stream).await {
    handle_request(&url, &mut tls_stream).await?;
  }
  else {
    tls_stream.write(Status::BadRequest.to_bytes()).await?;
  }

  tls_stream.flush().await?;
  Ok(())
}

async fn handle_request<W: Write + Unpin>(request_url: &Url, tls_stream: &mut W) -> Result<()> {
  let mut request_path = PathBuf::from("/var/gemini/");
  request_path.extend(request_url.path_segments().unwrap());
  println!("Request URL: {}", request_url);
  println!("Request path: {:?}", request_path);

  if request_path.is_dir().await {
    if request_url.as_str().ends_with("/") {
      request_path.push("index.gmi");
    }
    else {
      let mut redirect_url = String::from(request_url.as_str());
      redirect_url.push_str("/");
      write_header(tls_stream, Status::RedirectPermanent, &redirect_url).await?;
      return Ok(())
    }
  }
  
  if request_path.exists().await {
    let mut file = File::open(request_path).await?;
    write_header(tls_stream, Status::Success, "text/gemini").await?;
    async_std::io::copy(&mut file, tls_stream).await?;
  }
  else {
    write_header(tls_stream, Status::NotFound, "File not found").await?;
  }

  Ok(())
}

async fn parse_request<R: Read + Unpin>(tls_stream: &mut R) -> Result<Url> {
  let mut buffer = [0; 1026];
  let mut length = 0;

  loop {
    let read_count = tls_stream.read(&mut buffer[length..]).await?;
    length += read_count;
    if buffer[..length].ends_with(b"\r\n") {
      break;
    }
    else if read_count == 0 {
      Err("Request terminated unexpectedly")?
    }
  }

  let request = std::str::from_utf8(&buffer[..length - 2])?;
  let url = Url::parse(&request)?;
  println!("Request URL: {:?}", url);

  Ok(url)
}

async fn write_header<W: Write + Unpin>(stream: &mut W, status: Status, meta: &str) -> Result<()> {
  stream.write(status.to_bytes()).await?;
  stream.write(b" ").await?;
  stream.write(meta.as_bytes()).await?;
  stream.write(b"\r\n").await?;

  Ok(())
}