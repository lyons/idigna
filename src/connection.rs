use async_std::{
  fs, fs::File,
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
use percent_encoding::{
  CONTROLS,
  percent_decode_str,
  utf8_percent_encode,
};
use std::{
  error::Error,
  ffi::OsStr,
  marker::Unpin,
};
use url::Url;

use crate::config::ServerConfig;
use crate::status::Status;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub async fn handle_connection(acceptor: &TlsAcceptor, tcp_stream: &mut TcpStream, config: &ServerConfig) -> Result<()> {
  //let peer_addr = tcp_stream.peer_addr()?;
  //println!("Connection from {}", peer_addr);

  let handshake = acceptor.accept(tcp_stream);
  let mut tls_stream = handshake.await?;

  if let Ok(url) = parse_request(&mut tls_stream).await {
    handle_request(&url, &mut tls_stream, config).await?;
  }
  else {
    tls_stream.write(Status::BadRequest.to_bytes()).await?;
  }

  tls_stream.flush().await?;
  Ok(())
}

async fn handle_request<W: Write + Unpin>(request_url: &Url, stream: &mut W, config: &ServerConfig) -> Result<()> {
  let mut request_path = PathBuf::from(&config.server_root);
  let raw_url_path = if let Some(foo) = request_url.path().strip_prefix("/") {
    foo
  }
  else {
    request_url.path()
  };
  let unescaped_url_path = percent_decode_str(raw_url_path).decode_utf8()?;
  request_path.push(unescaped_url_path.as_ref());

  if request_path.is_dir().await {
    if !request_url.as_str().ends_with("/") 
    {
      let mut redirect_url = String::from(request_url.as_str());
      redirect_url.push_str("/");
      write_header(stream, Status::RedirectPermanent, &redirect_url).await?;
    }
    else {
      let mut index_found = false;
      for filename in &config.index {
        request_path.push(filename);
        if request_path.exists().await {
          index_found = true;
          break;
        }
        else {
          request_path.pop();
        }
      }

      if index_found {
        serve_file(request_path, stream).await?;
      }
      else {
        let generated_index = generate_autoindex(request_path, request_url).await?;
        write_header(stream, Status::Success, "text/gemini").await?;
        stream.write(generated_index.as_bytes()).await?;
        //write_header(stream, Status::TemporaryFailure, "Forbidden").await?;
      }
    }
  }
  
  else if request_path.exists().await {
    serve_file(request_path, stream).await?;
  }

  else {
    write_header(stream, Status::NotFound, "File not found").await?;
  }

  Ok(())
}

async fn serve_file<W: Write + Unpin>(path: PathBuf, stream: &mut W) -> Result<()> {
  let mimetype = get_mimetype(path.extension());
  let mut file = File::open(path).await?;
  write_header(stream, Status::Success, mimetype).await?;
  async_std::io::copy(&mut file, stream).await?;
  Ok(())
}

async fn parse_request<R: Read + Unpin>(stream: &mut R) -> Result<Url> {
  let mut buffer = [0; 1026];
  let mut length = 0;

  loop {
    let read_count = stream.read(&mut buffer[length..]).await?;
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

fn get_mimetype(extension: Option<&OsStr>) -> &'static str {
  match extension {
    Some(extension) => {
      if let Some(extension) = extension.to_str() {
        match extension {
          "gemini" => "text/gemini",
          "gmi"    => "text/gemini",
          "md"     => "text/markdown",
          "txt"    => "text/plain",
          _        => "application/octet-stream",
        }
      }
      else {
        "application/octet-stream"
      }
    },
    None => "text/plain",
  }
}

async fn generate_autoindex(path: PathBuf, base_url: &Url) -> Result<String> {
  let mut directories: Vec<String> = Vec::new();
  let mut files: Vec<String> = Vec::new();
  let mut result = String::new();
  let mut dir = fs::read_dir(path).await?;

  result.push_str("Index of ");
  result.push_str(base_url.as_str());
  result.push_str("\n\n");
  
  while let Some(entry) = dir.next().await {
    let entry = entry?;
    let path = entry.path();
    if let Some(filename) = path.file_name() {
      if let Some(filename) = filename.to_str() {
        if path.is_dir().await {
          let mut filename = filename.to_string();
          filename.push('/');
          directories.push(filename);
        }
        else {
          files.push(filename.to_string());
        }
      }
    }
  }

  directories.sort_by(|x, y| x.to_lowercase().cmp(&y.to_lowercase()));
  files.sort_by(|x, y| x.to_lowercase().cmp(&y.to_lowercase()));

  for d in directories {
    let url = base_url.join(&d)?;
    let escaped_url = utf8_percent_encode(url.as_str(), CONTROLS);

    result.push_str("=> ");
    result.push_str(&escaped_url.to_string());
    result.push_str(" ");
    result.push_str(&d);
    result.push_str("\n");
  }
  for f in files {
    let url = base_url.join(&f)?;
    let escaped_url = utf8_percent_encode(url.as_str(), CONTROLS);

    result.push_str("=> ");
    result.push_str(&escaped_url.to_string());
    result.push_str(" ");
    result.push_str(&f);
    result.push_str("\n");
  }

  Ok(result)
}
