use async_std::path::Path;
use mime_guess::{self, mime};

pub fn infer_mimetype(path: &Path) -> String {
  let extension = match path.extension() {
    Some(ext) => ext.to_str().unwrap(),
    // If no extension, probably a dotfile or something so assume text
    None => return String::from("text/plain"),
  };

  match extension {
    "gemini" => String::from("text/gemini"),
    "gmi"    => String::from("text/gemini"),
    _ => {
      let mime = mime_guess::from_ext(extension).first_or(mime::APPLICATION_OCTET_STREAM);
      let essence = mime.essence_str();
      String::from(essence)
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn is_gemini_file() {
    let foo = Path::new("foo.gmi");
    let bar = Path::new("bar.gemini");

    assert_eq!(infer_mimetype(foo), "text/gemini");
    assert_eq!(infer_mimetype(bar), "text/gemini");
  }

  #[test]
  fn dotfiles_are_text() {
    let foo = Path::new(".gitignore");
    
    assert_eq!(infer_mimetype(foo), "text/plain");
  }

  #[test]
  fn a_few_common_mimetypes() {
    let png = Path::new("foo.png");
    let mp3 = Path::new("bar.mp3");
    let xlsx = Path::new("baz.xlsx");

    assert_eq!(infer_mimetype(png), "image/png");
    assert_eq!(infer_mimetype(mp3), "audio/mpeg");
    assert_eq!(infer_mimetype(xlsx), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
  }

  #[test]
  fn unknown_is_octet_stream() {
    let foo = Path::new("foo.unknownextension");

    assert_eq!(infer_mimetype(foo), "application/octet-stream");
  }
}