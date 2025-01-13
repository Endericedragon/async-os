use alloc::string::String;
use async_std::collections::Vec;
use core::fmt;

/// Local type-alias for multihash.
///
/// Must be big enough to accommodate for `MAX_INLINE_KEY_LENGTH`.
/// 64 satisfies that and can hold 512 bit hashes which is what the ecosystem typically uses.
/// Given that this appears in our type-signature, using a "common" number here makes us more compatible.
pub type Multihash = multihash::Multihash<64>;

/// Public keys with byte-lengths smaller than `MAX_INLINE_KEY_LENGTH` will be
/// automatically used as the peer id using an identity multihash.
const MAX_INLINE_KEY_LENGTH: usize = 42;

const MULTIHASH_IDENTITY_CODE: u64 = 0;
const MULTIHASH_SHA256_CODE: u64 = 0x12;

/// Identifier of a peer of the network.
///
/// The data is a CIDv0 compatible multihash of the protobuf encoded public key of the peer
/// as specified in [specs/peer-ids](https://github.com/libp2p/specs/blob/master/peer-ids/peer-ids.md).
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PeerId {
    multihash: Multihash,
}

impl PeerId {
    /// Builds a `PeerId` from `Vec<u8>`.
    pub fn from_vec_u8(mh_code: u64, data: Vec<u8>) -> Self {
        Self {
            multihash: Multihash::wrap(mh_code, &data)
                .expect("64 byte multihash provides sufficient space"),
        }
    }

    /// Parses a `PeerId` from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<PeerId, ParseError> {
        PeerId::from_multihash(Multihash::from_bytes(data)?)
            .map_err(|mh| ParseError::UnsupportedCode(mh.code()))
    }

    /// Tries to turn a `Multihash` into a `PeerId`.
    ///
    /// If the multihash does not use a valid hashing algorithm for peer IDs,
    /// or the hash value does not satisfy the constraints for a hashed
    /// peer ID, it is returned as an `Err`.
    pub fn from_multihash(multihash: Multihash) -> Result<PeerId, Multihash> {
        match multihash.code() {
            MULTIHASH_SHA256_CODE => Ok(PeerId { multihash }),
            MULTIHASH_IDENTITY_CODE if multihash.digest().len() <= MAX_INLINE_KEY_LENGTH => {
                Ok(PeerId { multihash })
            }
            _ => Err(multihash),
        }
    }

    // /// Generates a random peer ID from a cryptographically secure PRNG.
    // ///
    // /// This is useful for randomly walking on a DHT, or for testing purposes.
    // #[cfg(feature = "rand")]
    // pub fn random() -> PeerId {
    //     let peer_id = rand::thread_rng().gen::<[u8; 32]>();
    //     PeerId {
    //         multihash: Multihash::wrap(0x0, &peer_id).expect("The digest size is never too large"),
    //     }
    // }

    /// Returns a raw bytes representation of this `PeerId`.
    pub fn to_bytes(self) -> Vec<u8> {
        self.multihash.to_bytes()
    }

    /// Returns a base-58 encoded string of this `PeerId`.
    pub fn to_base58(self) -> String {
        bs58::encode(self.to_bytes()).into_string()
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PeerId").field(&self.to_base58()).finish()
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_base58().fmt(f)
    }
}

/// Error when parsing a [`PeerId`] from string or bytes.
#[derive(Debug)]
pub enum ParseError {
    B58(bs58::decode::Error),
    UnsupportedCode(u64),
    InvalidMultihash(multihash::Error),
}

impl From<multihash::Error> for ParseError {
    fn from(value: multihash::Error) -> Self {
        ParseError::InvalidMultihash(value)
    }
}
