use async_std::{
  io,
  net::TcpListener,
  path::{Path, PathBuf},
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
  net::ToSocketAddrs,
  sync::Arc,
};
use structopt::StructOpt;

mod connection;
mod status;

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

pub struct ConfigSlug {
  index: Vec<String>,
  server_root: String,
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

fn main() -> Result<()> {
  let options = Options::from_args();

  let addr = options
    .addr
    .to_socket_addrs()?
    .next()
    .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
  
  let config = load_config(&options)?;

  let acceptor = TlsAcceptor::from(Arc::new(config));

  let slug = ConfigSlug {
    index: vec![String::from("index.gemini"), String::from("index.gmi")],
    server_root: String::from("/Users/lyons/Documents/"),
  };
  let config_slug = Arc::new(slug);

  task::block_on(async {
    let listener = TcpListener::bind(&addr).await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
      let acceptor = acceptor.clone();
      let config_slug = config_slug.clone();
      let mut stream = stream?;

      task::spawn(async move {
        let res = connection::handle_connection(&acceptor, &mut stream, &config_slug).await;
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
