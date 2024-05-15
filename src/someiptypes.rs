pub mod someip_types {
    pub type SomeipServiceId = u16;
    pub type SomeipMethodId = u16;
    pub type SomeipClientId = u16;
    pub type SomeipSessionId = u16;

    pub type PacketIndex = isize;

    pub enum SomeipTransportPortocol {
        UNDEFINED,
        TCP,
        UDP,
    }

}
