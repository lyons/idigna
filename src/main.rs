use async_std::{
  io,
  net::TcpListener,
  task,
};
use async_tls::TlsAcceptor;
use futures::{
  stream::StreamExt,
};
use log;
use simple_logger::SimpleLogger;
use std::{
  error::Error,
  net::ToSocketAddrs,
  path::PathBuf,
  sync::Arc,
};
use structopt::StructOpt;

mod config;
mod connection;
mod mimetype;
mod request;
mod status;
mod tls;

#[derive(StructOpt)]
struct Options {
  #[structopt(default_value = "config.json", parse(from_os_str))]
  configuration_path: PathBuf,
}

pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result<()> {
  SimpleLogger::new().with_level(log::LevelFilter::Debug).init()?;

  let options = Options::from_args();
  let (conf, servers) = config::load(options.configuration_path)?;
  let servers = Arc::new(servers);

  let addr = conf
    .listen_addr
    .to_socket_addrs()?
    .next()
    .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
  
  let tls_config = tls::load_tls_config(&conf)?;

  let acceptor = TlsAcceptor::from(Arc::new(tls_config));

  task::block_on(async {
    let listener = TcpListener::bind(&addr).await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
      let acceptor = acceptor.clone();
      let servers = servers.clone();
      let mut stream = stream?;

      task::spawn(async move {
        let res = connection::handle_connection(&acceptor, &mut stream, &servers).await;
        match res {
          Ok(_) => {}
          Err(err) => {
            log::warn!("Error handling connection: {:?}", err);
          }
        };
      });
    }

    Ok(())
  })
}
