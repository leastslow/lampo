#[allow(dead_code)]
pub enum HttpResponse {
  // Informational 1xx
  Continue,
  SwitchingProtocols,

  // Successful 2xx
  Ok,
  OkEstablished,
  Created,
  Accepted,
  NonAuthoritativeInformation,
  NoContent,
  ResetContent,
  PartialContent,

  // Redirection 3xx
  MultipleChoices,
  MovedPermanently,
  Found,
  SeeOther,
  NotModified,
  UseProxy,
  TemporaryRedirect,

  // Client Error 4xx
  BadRequest,
  Unauthorized,
  PaymentRequired,
  Forbidden,
  NotFound,
  MethodNotAllowed,
  NotAcceptable,
  ProxyAuthenticationRequired,
  RequestTimeout,
  Conflict,
  Gone,
  LengthRequired,
  PreconditionFailed,
  RequestEntityTooLarge,
  RequestUriTooLong,
  UnsupportedMediaType,
  RequestedRangeNotSatisfiable,
  ExpectationFailed,

  // Server Error 5xx
  InternalServerError,
  NotImplemented,
  BadGateway,
  ServiceUnavailable,
  GatewayTimeout,
  HttpVersionNotSupported,
}

impl HttpResponse {
  pub fn as_bytes(&self) -> &[u8] {
    match self {
      HttpResponse::Continue => b"HTTP/1.1 100 Continue\r\n\r\n",
      HttpResponse::SwitchingProtocols => b"HTTP/1.1 101 Switching Protocols\r\n\r\n",
      HttpResponse::Ok => b"HTTP/1.1 200 OK\r\n\r\n",
      HttpResponse::OkEstablished => b"HTTP/1.1 200 Connection Established\r\n\r\n",
      HttpResponse::Created => b"HTTP/1.1 201 Created\r\n\r\n",
      HttpResponse::Accepted => b"HTTP/1.1 202 Accepted\r\n\r\n",
      HttpResponse::NonAuthoritativeInformation => b"HTTP/1.1 203 Non-Authoritative Information\r\n\r\n",
      HttpResponse::NoContent => b"HTTP/1.1 204 No Content\r\n\r\n",
      HttpResponse::ResetContent => b"HTTP/1.1 205 Reset Content\r\n\r\n",
      HttpResponse::PartialContent => b"HTTP/1.1 206 Partial Content\r\n\r\n",
      HttpResponse::MultipleChoices => b"HTTP/1.1 300 Multiple Choices\r\n\r\n",
      HttpResponse::MovedPermanently => b"HTTP/1.1 301 Moved Permanently\r\n\r\n",
      HttpResponse::Found => b"HTTP/1.1 302 Found\r\n\r\n",
      HttpResponse::SeeOther => b"HTTP/1.1 303 See Other\r\n\r\n",
      HttpResponse::NotModified => b"HTTP/1.1 304 Not Modified\r\n\r\n",
      HttpResponse::UseProxy => b"HTTP/1.1 305 Use Proxy\r\n\r\n",
      HttpResponse::TemporaryRedirect => b"HTTP/1.1 307 Temporary Redirect\r\n\r\n",
      HttpResponse::BadRequest => b"HTTP/1.1 400 Bad Request\r\n\r\n",
      HttpResponse::Unauthorized => b"HTTP/1.1 401 Unauthorized\r\n\r\n",
      HttpResponse::PaymentRequired => b"HTTP/1.1 402 Payment Required\r\n\r\n",
      HttpResponse::Forbidden => b"HTTP/1.1 403 Forbidden\r\n\r\n",
      HttpResponse::NotFound => b"HTTP/1.1 404 Not Found\r\n\r\n",
      HttpResponse::MethodNotAllowed => b"HTTP/1.1 405 Method Not Allowed\r\n\r\n",
      HttpResponse::NotAcceptable => b"HTTP/1.1 406 Not Acceptable\r\n\r\n",
      HttpResponse::ProxyAuthenticationRequired => {
        b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"Leastslow Network\"\r\n\r\nAccess Denied"
      }
      HttpResponse::RequestTimeout => b"HTTP/1.1 408 Request Timeout\r\n\r\n",
      HttpResponse::Conflict => b"HTTP/1.1 409 Conflict\r\n\r\n",
      HttpResponse::Gone => b"HTTP/1.1 410 Gone\r\n\r\n",
      HttpResponse::LengthRequired => b"HTTP/1.1 411 Length Required\r\n\r\n",
      HttpResponse::PreconditionFailed => b"HTTP/1.1 412 Precondition Failed\r\n\r\n",
      HttpResponse::RequestEntityTooLarge => b"HTTP/1.1 413 Request Entity Too Large\r\n\r\n",
      HttpResponse::RequestUriTooLong => b"HTTP/1.1 414 Request-URI Too Long\r\n\r\n",
      HttpResponse::UnsupportedMediaType => b"HTTP/1.1 415 Unsupported Media Type\r\n\r\n",
      HttpResponse::RequestedRangeNotSatisfiable => b"HTTP/1.1 416 Requested Range Not Satisfiable\r\n\r\n",
      HttpResponse::ExpectationFailed => b"HTTP/1.1 417 Expectation Failed\r\n\r\n",
      HttpResponse::InternalServerError => b"HTTP/1.1 500 Internal Server Error\r\n\r\n",
      HttpResponse::NotImplemented => b"HTTP/1.1 501 Not Implemented\r\n\r\n",
      HttpResponse::BadGateway => b"HTTP/1.1 502 Bad Gateway\r\n\r\n",
      HttpResponse::ServiceUnavailable => b"HTTP/1.1 503 Service Unavailable\r\n\r\n",
      HttpResponse::GatewayTimeout => b"HTTP/1.1 504 Gateway Timeout\r\n\r\n",
      HttpResponse::HttpVersionNotSupported => b"HTTP/1.1 505 HTTP Version Not Supported\r\n\r\n",
    }
  }
}
