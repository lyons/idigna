use async_std::{
  fs, fs::File,
  io::{Read, Write},
  path::{Path, PathBuf},
  prelude::*,
};
use percent_encoding::{
  CONTROLS,
  percent_decode_str,
  utf8_percent_encode,
};
use regex::Regex;
use std::{
  error::Error,
  ffi::OsStr,
  marker::Unpin,
};
use url::Url;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

use crate::config::{
  RewriteRule,
  ServerConfig,
};
use crate::status::Status;

pub async fn parse<R: Read + Unpin>(stream: &mut R) -> Result<Url> {
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

  Ok(url)
}

pub async fn handle<W: Write + Unpin>(stream: &mut W, request_url: &Url, config: &ServerConfig) -> Result<()> {
  let url_path = percent_decode_str(request_url.path()).decode_utf8()?;
  let url_path = rewrite_url(&url_path, &config.rewrite_rules);
  
  // Redirect request if possible ----------------------------------------------
  if let Some(redirect) = apply_redirects(&url_path, &config.redirect_rules) {
    let mut redirect_url;
    if let Ok(url) = Url::parse(&redirect) {redirect_url = url;}
    else {
      redirect_url = request_url.clone();
      redirect_url.set_path(&redirect);
    }
    send_header(stream, Status::RedirectPermanent, redirect_url.as_str()).await?;
  }

  // Otherwise attempt to serve request ----------------------------------------
  else {
    // When building the path to attempt serving a file from, we need to strip the
    // leading '/' from the URL path (if it exists (which it always should)), as
    // pushing an absolute path on to a PathBuf replaces the existing path in it
    // rather than appending it. 
    let mut request_path = Path::new(&url_path);
    if request_path.is_absolute() {
      request_path = request_path.strip_prefix("/")?;
    }
    let mut file_path = PathBuf::from(&config.server_root);
    file_path.push(request_path);

    // Requested path is directory ---------------------------------------------
    if file_path.is_dir().await {
      if !request_url.as_str().ends_with("/") 
      {
        let mut redirect_url = String::from(request_url.as_str());
        redirect_url.push_str("/");
        send_header(stream, Status::RedirectPermanent, &redirect_url).await?;
      }
      else {
        let mut index_found = false;
        for filename in &config.index {
          file_path.push(filename);
          if file_path.exists().await {
            index_found = true;
            break;
          }
          else {
            file_path.pop();
          }
        }

        if index_found {
          send_file(stream, file_path).await?;
        }
        else {
          if is_auto_indexed(&url_path, &config.autoindex_rules) {
            let generated_index = generate_index(file_path, request_url).await?;
            send_header(stream, Status::Success, "text/gemini").await?;
            stream.write(generated_index.as_bytes()).await?;
          }
          else {
            send_header(stream, Status::TemporaryFailure, "Forbidden").await?;
          }
        }
      }
    }
    
    // Requested path is file --------------------------------------------------
    else if file_path.exists().await {
      send_file(stream, file_path).await?;
    }

    // Requested path does not exist -------------------------------------------
    else {
      send_header(stream, Status::NotFound, "File not found").await?;
    }
  }

  Ok(())
}

fn is_auto_indexed(url_path: &str, index_rules: &[Regex]) -> bool {
  for rule in index_rules {
    if rule.is_match(url_path) {return true}
  }

  false
}

fn rewrite_url(url_path: &str, rewrite_rules: &[RewriteRule]) -> String {
  let mut path = String::from(url_path);
  for rule in rewrite_rules {
    path = rule.pattern.replace(&path, rule.substitution.as_str()).into_owned();
    println!("Rewrite result: {:?}", path);
  }

  path
}

fn apply_redirects(url_path: &str, redirect_rules: &[RewriteRule]) -> Option<String> {
  for rule in redirect_rules {
    if rule.pattern.is_match(url_path) {
      let result = rule.pattern.replace(url_path, rule.substitution.as_str()).into_owned();
      return Some(result)
    }
  }

  None
}

async fn send_header<W: Write + Unpin>(stream: &mut W, status: Status, meta: &str) -> Result<()> {
  stream.write(status.to_bytes()).await?;
  stream.write(b" ").await?;
  stream.write(meta.as_bytes()).await?;
  stream.write(b"\r\n").await?;

  Ok(())
}

async fn send_file<W: Write + Unpin>(stream: &mut W, path: PathBuf) -> Result<()> {
  let mimetype = get_mimetype(path.extension());
  let mut file = File::open(path).await?;
  send_header(stream, Status::Success, mimetype).await?;
  async_std::io::copy(&mut file, stream).await?;
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

async fn generate_index(path: PathBuf, base_url: &Url) -> Result<String> {
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
