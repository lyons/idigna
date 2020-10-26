use regex::Regex;
use std::{
  fmt,
  str::FromStr,
};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Status {
  Input(Option<String>),
  SensitiveInput(Option<String>),
  Success(Option<String>),
  RedirectTemporary(Option<String>),
  RedirectPermanent(Option<String>),
  TemporaryFailure(Option<String>),
  ServerUnavailable(Option<String>),
  CgiError(Option<String>),
  ProxyError(Option<String>),
  SlowDown(Option<String>),
  PermanentFailure(Option<String>),
  NotFound(Option<String>),
  Gone(Option<String>),
  ProxyRequestRefused(Option<String>),
  BadRequest(Option<String>),
  ClientCertificateRequired(Option<String>),
  CertificateNotAuthorized(Option<String>),
  CertificateNotValid(Option<String>),
  Unknown(u16, Option<String>),
}

impl Status {
  pub fn new(code: u16, meta: &str) -> Self {
    let mut meta = String::from(meta);
    meta.truncate(1024);

    match code {
      10 => Status::Input(Some(meta)),
      11 => Status::SensitiveInput(Some(meta)),
      20 => Status::Success(Some(meta)),
      30 => Status::RedirectTemporary(Some(meta)),
      31 => Status::RedirectPermanent(Some(meta)),
      40 => Status::TemporaryFailure(Some(meta)),
      41 => Status::ServerUnavailable(Some(meta)),
      42 => Status::CgiError(Some(meta)),
      43 => Status::ProxyError(Some(meta)),
      44 => Status::SlowDown(Some(meta)),
      50 => Status::PermanentFailure(Some(meta)),
      51 => Status::NotFound(Some(meta)),
      52 => Status::Gone(Some(meta)),
      53 => Status::ProxyRequestRefused(Some(meta)),
      59 => Status::BadRequest(Some(meta)),
      60 => Status::ClientCertificateRequired(Some(meta)),
      61 => Status::CertificateNotAuthorized(Some(meta)),
      62 => Status::CertificateNotValid(Some(meta)),
      _  => Status::Unknown(code, Some(meta)),
    }
  }

  pub fn numerical_code(&self) -> u16 {
    match self {
      Status::Input(_) => 10,
      Status::SensitiveInput(_) => 11,
      Status::Success(_) => 20,
      Status::RedirectTemporary(_) => 30,
      Status::RedirectPermanent(_) => 31,
      Status::TemporaryFailure(_) => 40,
      Status::ServerUnavailable(_) => 41,
      Status::CgiError(_) => 42,
      Status::ProxyError(_) => 43,
      Status::SlowDown(_) => 44,
      Status::PermanentFailure(_) => 50,
      Status::NotFound(_) => 51,
      Status::Gone(_) => 52,
      Status::ProxyRequestRefused(_) => 53,
      Status::BadRequest(_) => 59,
      Status::ClientCertificateRequired(_) => 60,
      Status::CertificateNotAuthorized(_) => 61,
      Status::CertificateNotValid(_) => 62,
      Status::Unknown(code, _) => *code,
    }
  }

  pub fn canonical_reason(&self) -> &'static str {
    match self {
      Status::Input(_) => "INPUT",
      Status::SensitiveInput(_) => "SENSITIVE INPUT",
      Status::Success(_) => "SUCCESS",
      Status::RedirectTemporary(_) => "REDIRECT - TEMPORARY",
      Status::RedirectPermanent(_) => "REDIRECT - PERMANENT",
      Status::TemporaryFailure(_) => "TEMPORARY FAILURE",
      Status::ServerUnavailable(_) => "SERVER UNAVAILABLE",
      Status::CgiError(_) => "CGI ERROR",
      Status::ProxyError(_) => "PROXY ERROR",
      Status::SlowDown(_) => "SLOW DOWN",
      Status::PermanentFailure(_) => "PERMANENT FAILURE",
      Status::NotFound(_) => "NOT FOUND",
      Status::Gone(_) => "GONE",
      Status::ProxyRequestRefused(_) => "PROXY REQUEST REFUSED",
      Status::BadRequest(_) => "BAD REQUEST",
      Status::ClientCertificateRequired(_) => "CLIENT CERTIFICATE REQUIRED",
      Status::CertificateNotAuthorized(_) => "CERTIFICATE NOT AUTHORIZED",
      Status::CertificateNotValid(_) => "CERTIFICATE NOT VALID",
      Status::Unknown(_, _) => "UNKNOWN STATUS CODE",
    }
  }

  pub fn meta(&self) -> Option<&str> {
    let foo = match self {
      Status::Input(meta) => meta,
      Status::SensitiveInput(meta) => meta,
      Status::Success(meta) => meta,
      Status::RedirectTemporary(meta) => meta,
      Status::RedirectPermanent(meta) => meta,
      Status::TemporaryFailure(meta) => meta,
      Status::ServerUnavailable(meta) => meta,
      Status::CgiError(meta) => meta,
      Status::ProxyError(meta) => meta,
      Status::SlowDown(meta) => meta,
      Status::PermanentFailure(meta) => meta,
      Status::NotFound(meta) => meta,
      Status::Gone(meta) => meta,
      Status::ProxyRequestRefused(meta) => meta,
      Status::BadRequest(meta) => meta,
      Status::ClientCertificateRequired(meta) => meta,
      Status::CertificateNotAuthorized(meta) => meta,
      Status::CertificateNotValid(meta) => meta,
      Status::Unknown(_, meta) => meta,
    };

    match foo {
      Some(str) => Some(&str),
      None      => None,
    }
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    self.to_string().into_bytes()
  }
}

impl fmt::Display for Status {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let code = self.numerical_code();
    let meta = match self.meta() {
      Some(text) => text,
      None => self.canonical_reason(),
    };
    write!(f, "{} {}\r\n", code, meta)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ParseStatusError;

impl FromStr for Status {
  type Err = ParseStatusError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let re = Regex::new(r"^(\d{2}) (.{0,1024})\r\n").unwrap();

    match re.captures(s) {
      Some(caps) => {
        let code_str = caps.get(1).map_or("", |m| m.as_str());
        let meta_str = caps.get(2).map_or("", |m| m.as_str());
        if let Ok(code) = code_str.parse::<u16>() {
          Ok(Status::new(code, meta_str))
        }
        else {
          Err(ParseStatusError)
        }
      },
      None => Err(ParseStatusError),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_display() {
    assert_eq!(format!("{}", Status::new(20, "")),
               "20 SUCCESS");
    assert_eq!(format!("{}", Status::new(51, "Some blob of meta text")),
               "51 NOT FOUND");
    assert_eq!(format!("{}", Status::new(99, "")),
               "99 UNKNOWN STATUS CODE");
  }

  #[test]
  fn test_meta() {
    assert_eq!(Status::new(20, "").meta(),
               "");
    assert_eq!(Status::new(44, "Whoa there bucko, slow down!").meta(),
               "Whoa there bucko, slow down!");
  }

  #[test]
  fn test_from_str() {
    let valid_status  = "20 text/gemini\r\n"; // valid
    let missing_code  = "two hundred not ok\r\n"; // invalid
    let short_code    = "9 the status code should be two digits\r\n"; // invalid
    let long_code     = "200 not ok\r\n"; // invalid
    let empty_meta    = "51 \r\n"; //valid
    let missing_space = "51notfound\r\n"; // invalid
    let with_tab      = "51\tNot found!\r\n"; // invalid
    let long_meta     = &format!("51 {}\r\n", String::from_utf8(vec![b'f'; 1024]).unwrap()); // valid
    let too_long_meta = &format!("51 {}\r\n", String::from_utf8(vec![b'f'; 1025]).unwrap()); // invalid
    let missing_cr    = "20 text/gemini\n"; // invalid
    let missing_lf    = "20 text/gemini\r"; // invalid

    assert_eq!(Status::from_str(valid_status).unwrap(),
               Status::new(20, "text/gemini"));
    assert_eq!(Status::from_str(empty_meta).unwrap(),
               Status::new(51, ""));
    assert_eq!(Status::from_str(long_meta).unwrap(),
               Status::new(51, &String::from_utf8(vec![b'f'; 1024]).unwrap()));
    
    assert!(Status::from_str(missing_code).is_err());
    assert!(Status::from_str(short_code).is_err());
    assert!(Status::from_str(long_code).is_err());
    assert!(Status::from_str(missing_space).is_err());
    assert!(Status::from_str(with_tab).is_err());
    assert!(Status::from_str(too_long_meta).is_err());
    assert!(Status::from_str(missing_cr).is_err());
    assert!(Status::from_str(missing_lf).is_err());
  }
}