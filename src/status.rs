use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Status {
  Input,
  SensitiveInput,
  Success,
  RedirectTemporary,
  RedirectPermanent,
  TemporaryFailure,
  ServerUnavailable,
  CgiError,
  ProxyError,
  SlowDown,
  PermanentFailure,
  NotFound,
  Gone,
  ProxyRequestRefused,
  BadRequest,
  ClientCertificateRequired,
  CertificateNotAuthorized,
  CertificateNotValid,
}

impl Status {
  pub fn code(&self) -> u16 {
    match self {
      Status::Input => 10,
      Status::SensitiveInput => 11,
      Status::Success => 20,
      Status::RedirectTemporary => 30,
      Status::RedirectPermanent => 31,
      Status::TemporaryFailure => 40,
      Status::ServerUnavailable => 41,
      Status::CgiError => 42,
      Status::ProxyError => 43,
      Status::SlowDown => 44,
      Status::PermanentFailure => 50,
      Status::NotFound => 51,
      Status::Gone => 52,
      Status::ProxyRequestRefused => 53,
      Status::BadRequest => 59,
      Status::ClientCertificateRequired => 60,
      Status::CertificateNotAuthorized => 61,
      Status::CertificateNotValid => 62,
    }
  }

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
    write!(f, "{}", self.code())
  }
}
