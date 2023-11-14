use std::time::Duration;

use bytes::BytesMut;
use httparse::Header;
use tokio::{io::AsyncReadExt, net::TcpStream};
use url::Url;

use crate::utils;

use super::utils::error::HttpParserError;

pub struct HttpParser<'a> {
  stream: &'a mut TcpStream,
  buffer: BytesMut,
}

pub struct HttpRequestData {
  pub path: String,
  pub method: String,
  pub version: String,
  pub host: (String, u16),
  pub headers: BytesMut,
  pub body: BytesMut,
  pub authentication: Option<(String, String)>,
}

impl HttpRequestData {
  pub fn new(
    path: &str,
    method: &str,
    version: &str,
    headers: BytesMut,
    body: BytesMut,
    host: (String, u16),
    authentication: Option<(String, String)>,
  ) -> Self {
    Self {
      path: path.to_string(),
      method: method.to_string(),
      version: version.to_string(),
      host,
      headers,
      body,
      authentication,
    }
  }

  pub fn into_request(&mut self) -> BytesMut {
    let mut request = BytesMut::new();
    let path = self.convert_absolute_uri_to_path().unwrap_or(self.path.clone());

    // Start line
    request.extend_from_slice(format!("{} {} HTTP/{}\r\n", self.method, path, self.version).as_bytes());

    // Headers
    request.extend_from_slice(&self.headers);

    if self.body.is_empty() {
      self.body.extend_from_slice(b"\r\n");
    }
    // Rest of the request, trailing headers, extensions or body all in one chunk.
    request.extend_from_slice(&self.body);

    request
  }

  fn convert_absolute_uri_to_path(&self) -> Option<String> {
    if self.path.starts_with("http://") || self.path.starts_with("https://") {
      if let Ok(url) = Url::parse(&self.path) {
        let mut path = url.path().to_string();
        if path.is_empty() {
          path = String::from("/");
        }
        if let Some(query) = url.query() {
          path.push('?');
          path.push_str(query);
        }
        return Some(path);
      }
    }

    return None;
  }
}

impl<'a> HttpParser<'a> {
  const MAX_BUF: usize = 16384;
  const MAX_TIMEOUT: Duration = Duration::from_secs(10);

  pub fn new(stream: &mut TcpStream) -> HttpParser {
    HttpParser {
      stream,
      buffer: BytesMut::with_capacity(HttpParser::MAX_BUF / 2),
    }
  }

  pub async fn read(&mut self) -> Result<HttpRequestData, HttpParserError> {
    loop {
      match tokio::time::timeout(HttpParser::MAX_TIMEOUT, self.stream.read_buf(&mut self.buffer)).await {
        Ok(result) => {
          match result {
            Ok(bytes_read) => {
              if bytes_read == 0 {
                // Client closed the connection.
                break;
              }
              if self.buffer.len() > HttpParser::MAX_BUF {
                return Err(HttpParserError::BufferLimitExceeded);
              }

              let request = self.parse_request().await;

              self.buffer.clear();

              return request;
            }
            Err(e) => return Err(HttpParserError::StreamReadError(e)),
          }
        }
        Err(_) => return Err(HttpParserError::StreamReadTimeout),
      }
    }
    Err(HttpParserError::ClosedConnection)
  }

  async fn parse_request(&mut self) -> Result<HttpRequestData, HttpParserError> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    match req.parse(&self.buffer) {
      Ok(httparse::Status::Complete(len)) => {
        let method = req.method.ok_or(HttpParserError::MissingMethod)?;
        let path = req.path.ok_or(HttpParserError::MissingPath)?;
        let version_u8 = req.version.ok_or(HttpParserError::MissingVersion)?;

        let (host_header, proxy_authorization_header, headers) = self.parse_headers(&req.headers).await?;
        let host = self.parse_host(method, path, host_header).ok_or(HttpParserError::MissingHost)?;

        let body = BytesMut::from(&self.buffer[len..]);
        let version = self.parse_version(version_u8)?;

        return Ok(HttpRequestData::new(
          path,
          method,
          version,
          headers,
          body,
          host,
          self.parse_auth_header(proxy_authorization_header).await,
        ));
      }
      Ok(httparse::Status::Partial) => (),
      Err(_) => {
        return Err(HttpParserError::Unknown);
      }
    }

    return Err(HttpParserError::Unknown);
  }

  fn parse_version(&self, version: u8) -> Result<&str, HttpParserError> {
    match version {
      0 => Ok("1.0"),
      1 => Ok("1.1"),
      _ => Err(HttpParserError::InvalidVersion),
    }
  }

  async fn parse_headers(&self, headers: &[Header<'a>]) -> Result<(Option<Header>, Option<Header>, BytesMut), HttpParserError> {
    let mut host: Option<Header> = None;
    let mut proxy_authorization: Option<Header> = None;

    let mut buf = BytesMut::new();

    for header in headers {
      if header.name.to_string().eq_ignore_ascii_case("host") {
        host = Some(header.to_owned());
      }
      if header.name.to_string().eq_ignore_ascii_case("proxy-authorization") {
        proxy_authorization = Some(header.to_owned());
        // We do not want to add the proxy-authorization header to the actual outgoing request.
        continue;
      }

      buf.extend_from_slice(header.name.as_bytes());
      buf.extend_from_slice(b": ");
      buf.extend_from_slice(header.value);
      buf.extend_from_slice(b"\r\n");
    }

    Ok((host, proxy_authorization, buf))
  }

  async fn parse_auth_header(&self, auth_header: Option<Header<'a>>) -> Option<(String, String)> {
    let auth_header_value = String::from_utf8(auth_header?.value.to_owned()).ok()?;

    match utils::auth::parse_credentials(&auth_header_value) {
      Some(a) => Some(a),
      None => {
        debug!("Failed to parse authentication header. Raw = {:?}", auth_header_value);
        return None;
      }
    }
  }

  fn parse_host(&self, method: &str, path: &str, host_header: Option<Header>) -> Option<(String, u16)> {
    // Special handling for CONNECT requests
    if method == "CONNECT" {
      let parts: Vec<&str> = path.splitn(2, ':').collect();
      let host = parts.get(0)?;
      let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(443);
      return Some((host.to_string(), port));
    }

    // Checking for scheme in the path
    let default_port = if path.starts_with("http://") {
      80
    } else if path.starts_with("https://") {
      443
    } else {
      // If path does not start with any scheme, try Host header as fallback
      if let Ok(host_header_str) = String::from_utf8(host_header?.value.to_owned()) {
        if let Ok(url) = Url::parse(&format!("http://{}", host_header_str)) {
          // Add "http://" just for parsing purposes.
          return Some((url.host_str()?.to_string(), url.port().unwrap_or(80)));
        }
        return None;
      } else {
        return None;
      }
    };

    if let Ok(url) = Url::parse(path) {
      Some((url.host_str()?.to_string(), url.port().unwrap_or(default_port)))
    } else {
      None
    }
  }
}
