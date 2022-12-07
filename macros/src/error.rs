use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidExpression {
  #[error("more than one leading capture")]
  InvalidCapture,
}

#[derive(Debug, Error)]
pub enum CaptureError {
  #[error("path starts with unexpected leading colons")]
  UnexpectedLeadingColon,
  #[error("unexpected path segment: {0}")]
  UnexpectedPathSegment(syn::Ident),
  #[error("provided path ({0:?}) doesn't match the expression")]
  NoMatch(syn::Path),
}

#[derive(Debug, Error)]
pub enum ParseError {
  #[error("unable to parse path: {0}")]
  PathParseError(String),
}
