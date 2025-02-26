use bytes::{BufMut, Bytes, BytesMut};
use core::error::Error;
use core::fmt;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
};
use async_collections::vec::Vec;

use super::Version;

/// 备选协议的最大数量。
const MAX_PROTOCOLS: usize = 1000;
/// multistream-select 1.0.0 的header信息，但被编码为字节数组。
const MSG_MULTISTREAM_1_0: &[u8] = b"/multistream/1.0.0\n";
/// multistream-select的NotAvailable消息, 但被编码为字节数组。
const MSG_PROTOCOL_NA: &[u8] = b"na\n";
/// multistream-select的'ls'消息, 但被编码为字节数组。
const MSG_LS: &[u8] = b"ls\n";

/// The multistream-select header lines preceeding negotiation.
///
/// Every [`Version`] has a corresponding header line.
/// HeaderLine可以理解为某个协议类。不管是传统V1还是V1Lazy，它们都属于V1这个HeaderLine。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum HeaderLine {
    /// The `/multistream/1.0.0` header line.
    V1,
}

impl From<Version> for HeaderLine {
    fn from(v: Version) -> HeaderLine {
        match v {
            Version::V1 | Version::V1Lazy => HeaderLine::V1,
        }
    }
}

/// A protocol (name) exchanged during protocol negotiation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Protocol(String);
impl AsRef<str> for Protocol {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<Bytes> for Protocol {
    type Error = ProtocolError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if !value.as_ref().starts_with(b"/") {
            return Err(ProtocolError::InvalidProtocol);
        }
        let protocol_as_string =
            String::from_utf8(value.to_vec()).map_err(|_| ProtocolError::InvalidProtocol)?;

        Ok(Protocol(protocol_as_string))
    }
}

impl TryFrom<&[u8]> for Protocol {
    type Error = ProtocolError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(Bytes::copy_from_slice(value))
    }
}

impl TryFrom<&str> for Protocol {
    type Error = ProtocolError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !value.starts_with('/') {
            return Err(ProtocolError::InvalidProtocol);
        }

        Ok(Protocol(value.to_owned()))
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A multistream-select protocol message.
///
/// Multistream-select protocol messages are exchanged with the goal
/// of agreeing on a application-layer protocol to use on an I/O stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Message {
    /// A header message identifies the multistream-select protocol
    /// that the sender wishes to speak.
    Header(HeaderLine),
    /// A protocol message identifies a protocol request or acknowledgement.
    Protocol(Protocol),
    /// A message through which a peer requests the complete list of
    /// supported protocols from the remote.
    ListProtocols,
    /// A message listing all supported protocols of a peer.
    Protocols(Vec<Protocol>),
    /// A message signaling that a requested protocol is not available.
    NotAvailable,
}

impl Message {
    /// Encodes a `Message` into its byte representation.
    pub fn encode(&self, dest: &mut BytesMut) -> Result<(), ProtocolError> {
        match self {
            Message::Header(HeaderLine::V1) => {
                dest.reserve(MSG_MULTISTREAM_1_0.len());
                dest.put(MSG_MULTISTREAM_1_0);
                Ok(())
            }
            Message::Protocol(p) => {
                let len = p.as_ref().len() + 1; // + 1 for \n
                dest.reserve(len);
                dest.put(p.0.as_ref());
                dest.put_u8(b'\n');
                Ok(())
            }
            Message::ListProtocols => {
                dest.reserve(MSG_LS.len());
                dest.put(MSG_LS);
                Ok(())
            }
            Message::Protocols(ps) => {
                let mut buf = unsigned_varint::encode::usize_buffer();
                let mut encoded: Vec<u8> = Vec::with_capacity(ps.len());
                for p in ps {
                    encoded.extend(unsigned_varint::encode::usize(p.as_ref().len() + 1, &mut buf)); // +1 for '\n'
                    encoded.extend_from_slice(p.0.as_ref());
                    encoded.push(b'\n')
                }
                encoded.push(b'\n');
                dest.reserve(encoded.len());
                dest.put(encoded.as_ref());
                Ok(())
            }
            Message::NotAvailable => {
                dest.reserve(MSG_PROTOCOL_NA.len());
                dest.put(MSG_PROTOCOL_NA);
                Ok(())
            }
        }
    }

    /// Decodes a `Message` from its byte representation.
    pub fn decode(mut msg: Bytes) -> Result<Message, ProtocolError> {
        if msg == MSG_MULTISTREAM_1_0 {
            return Ok(Message::Header(HeaderLine::V1));
        }

        if msg == MSG_PROTOCOL_NA {
            return Ok(Message::NotAvailable);
        }

        if msg == MSG_LS {
            return Ok(Message::ListProtocols);
        }

        // If it starts with a `/`, ends with a line feed without any
        // other line feeds in-between, it must be a protocol name.
        if msg.first() == Some(&b'/')
            && msg.last() == Some(&b'\n')
            && !msg[..msg.len() - 1].contains(&b'\n')
        {
            let p = Protocol::try_from(msg.split_to(msg.len() - 1))?;
            return Ok(Message::Protocol(p));
        }

        // At this point, it must be an `ls` response, i.e. one or more
        // length-prefixed, newline-delimited protocol names.
        let mut protocols = Vec::new();
        let mut remaining: &[u8] = &msg;
        loop {
            // A well-formed message must be terminated with a newline.
            if remaining == [b'\n'] {
                break;
            } else if protocols.len() == MAX_PROTOCOLS {
                return Err(ProtocolError::TooManyProtocols);
            }

            // Decode the length of the next protocol name and check that
            // it ends with a line feed.
            let (len, tail) = unsigned_varint::decode::usize(remaining)?;
            if len == 0 || len > tail.len() || tail[len - 1] != b'\n' {
                return Err(ProtocolError::InvalidMessage);
            }

            // Parse the protocol name.
            let p = Protocol::try_from(Bytes::copy_from_slice(&tail[..len - 1]))?;
            protocols.push(p);

            // Skip ahead to the next protocol.
            remaining = &tail[len..];
        }

        Ok(Message::Protocols(protocols))
    }
}

/// A protocol error.
#[derive(Debug)]
pub enum ProtocolError {
    /// I/O error.
    // IoError(io::Error),
    IoError(async_io::IoError),

    /// Received an invalid message from the remote.
    InvalidMessage,

    /// A protocol (name) is invalid.
    InvalidProtocol,

    /// Too many protocols have been returned by the remote.
    TooManyProtocols,
}

impl From<async_io::IoError> for ProtocolError {
    fn from(err: async_io::IoError) -> ProtocolError {
        ProtocolError::IoError(err)
    }
}

impl From<ProtocolError> for async_io::IoError {
    fn from(err: ProtocolError) -> Self {
        if let ProtocolError::IoError(e) = err {
            return e;
        }
        // io::ErrorKind::InvalidData.into()
        async_io::Error::InvalidData.into()
    }
}

impl From<unsigned_varint::decode::Error> for ProtocolError {
    fn from(err: unsigned_varint::decode::Error) -> ProtocolError {
        // Self::from(io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
        Self::from(async_io::IoError::new(
            async_io::Error::InvalidData,
            err.to_string(),
        ))
    }
}

impl Error for ProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ProtocolError::IoError(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ProtocolError::IoError(e) => write!(fmt, "I/O error: {e}"),
            ProtocolError::InvalidMessage => write!(fmt, "Received an invalid message."),
            ProtocolError::InvalidProtocol => write!(fmt, "A protocol (name) is invalid."),
            ProtocolError::TooManyProtocols => write!(fmt, "Too many protocols received."),
        }
    }
}
