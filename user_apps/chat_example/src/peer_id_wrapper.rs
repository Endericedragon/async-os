use libp2p::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct PeerIdWrapper(PeerId);

// 序列化 PeerIdWrapper ，以实现在网络中传输
impl Serialize for PeerIdWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.to_bytes().as_slice())
    }
}
// 反序列化 PeerIdWrapper ，以实现在网络中传输
impl<'de> Deserialize<'de> for PeerIdWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PeerIdVisitor;
        impl<'de> serde::de::Visitor<'de> for PeerIdVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A byte arraw representing a PeerId")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.to_vec())
            }
        }

        let bytes = deserializer.deserialize_bytes(PeerIdVisitor)?;
        PeerId::from_bytes(&bytes)
            .map(|inner| PeerIdWrapper(inner))
            .map_err(|e| serde::de::Error::custom(format!("Failed to deserialize PeerId: {e}")))
    }
}

// 方便地在本体和包装结构体之间进行转换
impl From<PeerId> for PeerIdWrapper {
    fn from(val: PeerId) -> Self {
        Self(val)
    }
}

impl From<&PeerId> for PeerIdWrapper {
    fn from(value: &PeerId) -> Self {
        Self(value.clone())
    }
}

impl From<&[u8]> for PeerIdWrapper {
    fn from(val: &[u8]) -> Self {
        PeerId::from_bytes(val).unwrap().into()
    }
}

impl PeerIdWrapper {
    pub fn into_inner(self) -> PeerId {
        self.0
    }

    pub fn as_inner(&self) -> PeerId {
        self.0
    }
}
