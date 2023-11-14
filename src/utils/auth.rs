use base64::engine::general_purpose;
use base64::Engine;

pub fn parse_credentials(header_value: &str) -> Option<(String, String)> {
  const PREFIX: &str = "Basic ";

  if !header_value.starts_with(PREFIX) {
    return None;
  }

  let encoded = &header_value[PREFIX.len()..];
  extract_credentials(encoded)
}

fn extract_credentials(encoded: &str) -> Option<(String, String)> {
  let decoded_bytes = general_purpose::STANDARD.decode(encoded).ok()?;
  let decoded_str = String::from_utf8(decoded_bytes).ok()?;

  let mut parts = decoded_str.splitn(2, ':');
  let username = parts.next()?.to_string();
  let password = parts.next()?.to_string();

  if username.is_empty() || password.is_empty() {
    return None;
  }

  Some((username, password))
}
