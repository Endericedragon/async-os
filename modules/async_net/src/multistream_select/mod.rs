extern crate alloc;

mod message_io;
mod protocol;

/// Supported multistream-select versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Version {
    /// Version 1 of the multistream-select protocol. See [1] and [2].
    ///
    /// [1]: https://github.com/libp2p/specs/blob/master/connections/README.md#protocol-negotiation
    /// [2]: https://github.com/multiformats/multistream-select
    #[default]
    V1,
    /// A "lazy" variant of version 1 that is identical on the wire but whereby
    /// the dialer delays flushing protocol negotiation data in order to combine
    /// it with initial application data, thus performing 0-RTT negotiation.
    ///
    /// This strategy is only applicable for the node with the role of "dialer"
    /// in the negotiation and only if the dialer supports just a single
    /// application protocol. In that case the dialer immedidately "settles"
    /// on that protocol, buffering the negotiation messages to be sent
    /// with the first round of application protocol data (or an attempt
    /// is made to read from the `Negotiated` I/O stream).
    ///
    /// A listener will behave identically to `V1`. This ensures interoperability with `V1`.
    /// Notably, it will immediately send the multistream header as well as the protocol
    /// confirmation, resulting in multiple frames being sent on the underlying transport.
    /// Nevertheless, if the listener supports the protocol that the dialer optimistically
    /// settled on, it can be a 0-RTT negotiation.
    ///
    /// > **Note**: `V1Lazy` is specific to `rust-libp2p`. The wire protocol is identical to `V1`
    /// > and generally interoperable with peers only supporting `V1`. Nevertheless, there is a
    /// > pitfall that is rarely encountered: When nesting multiple protocol negotiations, the
    /// > listener should either be known to support all of the dialer's optimistically chosen
    /// > protocols or there is must be no intermediate protocol without a payload and none of
    /// > the protocol payloads must have the potential for being mistaken for a multistream-select
    /// > protocol message. This avoids rare edge-cases whereby the listener may not recognize
    /// > upgrade boundaries and erroneously process a request despite not supporting one of
    /// > the intermediate protocols that the dialer committed to. See [1] and [2].
    ///
    /// [1]: https://github.com/multiformats/go-multistream/issues/20
    /// [2]: https://github.com/libp2p/rust-libp2p/pull/1212
    V1Lazy,
    // Draft: https://github.com/libp2p/specs/pull/95
    // V2,
}
