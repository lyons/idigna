use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
  Input = 10,
  SensitiveInput = 11,
  Success = 20,
  RedirectTemporary = 30,
  RedirectPermanent = 31,
  TemporaryFailure = 40,
  ServerUnavailable = 41,
  CgiError = 42,
  ProxyError = 43,
  SlowDown = 44,
  PermanentFailure = 50,
  NotFound = 51,
  Gone = 52,
  ProxyRequestRefused = 53,
  BadRequest = 59,
  ClientCertificateRequired = 60,
  CertificateNotAuthorized = 61,
  CertificateNotValid = 62,
}

impl Status {
  pub fn to_bytes(&self) -> &'static [u8] {
    match self {
      Status::Input => b"10",
      Status::SensitiveInput => b"11",
      Status::Success => b"20",
      Status::RedirectTemporary => b"30",
      Status::RedirectPermanent => b"31",
      Status::TemporaryFailure => b"40",
      Status::ServerUnavailable => b"41",
      Status::CgiError => b"42",
      Status::ProxyError => b"43",
      Status::SlowDown => b"44",
      Status::PermanentFailure => b"50",
      Status::NotFound => b"51",
      Status::Gone => b"52",
      Status::ProxyRequestRefused => b"53",
      Status::BadRequest => b"59",
      Status::ClientCertificateRequired => b"60",
      Status::CertificateNotAuthorized => b"61",
      Status::CertificateNotValid => b"62",
    }
  }
}

impl fmt::Display for Status {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", *self as u16)
  }
}
