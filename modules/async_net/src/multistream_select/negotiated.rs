use super::protocol::ProtocolError;

pub enum NegotiationError {
    ProtocolError(ProtocolError),
    Failed,
}

impl From<ProtocolError> for NegotiationError {
    fn from(value: ProtocolError) -> Self {
        Self::ProtocolError(value)
    }
}