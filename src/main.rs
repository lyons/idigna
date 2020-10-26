use async_std::{
  io,
  io::{Read, Write},
  net::{
    TcpListener,
    TcpStream,
  },
  prelude::*,
  task,
};
use async_tls::TlsAcceptor;
use futures::{
  stream::StreamExt,
};
use rustls::{
  internal::pemfile::{
    certs,
    pkcs8_private_keys,
  },
  Certificate,
  NoClientAuth,
  PrivateKey,
  ServerConfig,
};
use std::{
  error::Error,
  fs::File,
  io::BufReader,
  marker::Unpin,
  net::ToSocketAddrs,
  path::{Path, PathBuf},
  sync::Arc,
};
use structopt::StructOpt;
use url::Url;

mod status;
use status::Status;


#[derive(StructOpt)]
struct Options {
  addr: String,
  
  #[structopt(short = "c", long = "cert", parse(from_os_str))]
  cert: PathBuf,

  #[structopt(short = "k", long = "key", parse(from_os_str))]
  key: PathBuf,

  // #[structopt(short = "d", long = "dir", parse(from_os_str))]
  // dir: PathBuf,
}

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
  certs(&mut BufReader::new(File::open(path)?))
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid certificate"))
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
  pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid key"))
}

fn load_config(options: &Options) -> Result<ServerConfig> {
  let certs = load_certs(&options.cert)?;
  let mut keys = load_keys(&options.key)?;

  let mut config = ServerConfig::new(NoClientAuth::new());
  config
    .set_single_cert(certs, keys.remove(0))
    .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

  Ok(config)
}

async fn handle_connection(acceptor: &TlsAcceptor, tcp_stream: &mut TcpStream) -> Result<()> {
  let peer_addr = tcp_stream.peer_addr()?;
  println!("Connection from {}", peer_addr);

  let handshake = acceptor.accept(tcp_stream);
  let mut tls_stream = handshake.await?;

  if let Ok(url) = parse_request(&mut tls_stream).await {
    handle_request(&url, &mut tls_stream).await?;
  }
  else {
    tls_stream.write(&Status::BadRequest(None).to_bytes()).await?;
    tls_stream.flush().await?;
  }

  Ok(())
}

async fn handle_request<W: Write + Unpin>(request_url: &Url, tls_stream: &mut W) -> Result<()> {
  let mut request_path = PathBuf::from("/var/gemini/");
  request_path.extend(request_url.path_segments().unwrap());
  println!("Request URL: {}", request_url);
  println!("Request path: {:?}", request_path);

  if request_path.is_dir() {
    if request_url.as_str().ends_with("/") {
      request_path.push("index.gmi");
    }
    else {
      let mut redirect_url = String::from(request_url.as_str());
      redirect_url.push_str("/");
      let response = Status::new(31, &redirect_url);
      tls_stream.write(&response.to_bytes()).await?;
      tls_stream.flush().await?;
      return Ok(())
    }
  }
  
  if request_path.exists() {
    //let mut file = File::open(request_path).unwrap();
    //let mut contents = String::new();
    tls_stream.write(&Status::new(20, "text/gemini").to_bytes()).await?;
    tls_stream.write(b"dummy content").await?;
    tls_stream.flush().await?;

    Ok(())
  }
  else {
    tls_stream.write(&Status::NotFound(None).to_bytes()).await?;
    tls_stream.flush().await?;
    Ok(())
  }
}

async fn parse_request<R: Read + Unpin>(tls_stream: &mut R) -> Result<Url> {
  let mut buffer = [0; 1029];
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
  let url = Url::parse(& request)?;
  println!("Request URL: {:?}", url);

  Ok(url)
}

fn main() -> Result<()> {
  let options = Options::from_args();

  let addr = options
    .addr
    .to_socket_addrs()?
    .next()
    .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
  
  let config = load_config(&options)?;

  let acceptor = TlsAcceptor::from(Arc::new(config));

  task::block_on(async {
    let listener = TcpListener::bind(&addr).await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
      let acceptor = acceptor.clone();
      let mut stream = stream?;

      task::spawn(async move {
        let res = handle_connection(&acceptor, &mut stream).await;
        match res {
          Ok(_) => {}
          Err(err) => {
            eprintln!("{:?}", err);
          }
        };
      });
    }

    Ok(())
  })
}
