use std::{
  cmp::Ordering,
  fmt,
  error::Error,
  str::FromStr,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatusCode(u8);

#[derive(Debug)]
pub struct InvalidStatusCode {
  _priv: (),
}

impl StatusCode {
  pub fn from_u8(src: u8) -> Result<StatusCode, InvalidStatusCode> {
    if src < 10 || src > 69 {
      Err(InvalidStatusCode::new())
    }
    else {
      Ok(StatusCode(src))
    }
  }

  pub fn as_u8(&self) -> u8 {
    self.0
  }

  // pub fn as_str(&self) -> &str {
  //   let str = format!("{}", self.0);
  //   & str
  // }

  pub fn canonical_reason(&self) -> Option<&'static str> {
    match self.0 {
      10 => Some("INPUT"),
      11 => Some("SENSITIVE INPUT"),
      20 => Some("SUCCESS"),
      30 => Some("REDIRECT - TEMPORARY"),
      31 => Some("REDIRECT - PERMANENT"),
      40 => Some("TEMPORARY FAILURE"),
      41 => Some("SERVER UNAVAILABLE"),
      42 => Some("CGI ERROR"),
      43 => Some("PROXY ERROR"),
      44 => Some("SLOW DOWN"),
      50 => Some("PERMANENT FAILURE"),
      51 => Some("NOT FOUND"),
      52 => Some("GONE"),
      53 => Some("PROXY REQUEST REFUSED"),
      59 => Some("BAD REQUEST"),
      60 => Some("CLIENT CERTIFICATE REQUIRED"),
      61 => Some("CERTIFICATE NOT AUTHORIZED"),
      62 => Some("CERTIFICATE NOT VALID"),
      _  => None,
    }
  }
}

impl fmt::Display for StatusCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {}", self.0, self.canonical_reason().unwrap_or("UNRECOGNIZED STATUS CODE"))
  }
}

impl PartialEq<u8> for StatusCode {
  fn eq(&self, other: &u8) -> bool {
    self.as_u8() == *other
  }
}

impl PartialEq<StatusCode> for u8 {
  fn eq(&self, other: &StatusCode) -> bool {
    other.as_u8() == *self
  }
}

impl PartialOrd<u8> for StatusCode {
  fn partial_cmp(&self, other: &u8) -> Option<Ordering> {
    self.as_u8().partial_cmp(other)
  }
}

impl PartialOrd<StatusCode> for u8 {
  fn partial_cmp(&self, other: &StatusCode) -> Option<Ordering> {
    self.partial_cmp(&other.as_u8())
  }
}

impl InvalidStatusCode {
  pub fn new() -> InvalidStatusCode {
    InvalidStatusCode {_priv: ()}
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_partial_ord() {
    let status = StatusCode::from_u8(20).unwrap();
    assert!(status < 40);
  }

  #[test]
  fn test_display() {
    let status = StatusCode::from_u8(20).unwrap();
    assert!(format!("{}", status) == "20 SUCCESS");
  }
}