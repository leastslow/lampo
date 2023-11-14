use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpParserError {
  #[error("missing http request version")]
  MissingVersion,
  #[error("invalid http request version")]
  InvalidVersion,
  #[error("missing http request method")]
  MissingMethod,
  #[error("missing http request target host")]
  MissingHost,
  #[error("missing http request path")]
  MissingPath,
  #[error("exceeded buffer limit during read")]
  BufferLimitExceeded,
  #[error("error while reading from stream. Err = {0}")]
  StreamReadError(std::io::Error),
  #[error("connection closed by the client")]
  ClosedConnection,
  #[error("stream read timeout")]
  StreamReadTimeout,
  #[error("unknown http parser error")]
  Unknown,
}
