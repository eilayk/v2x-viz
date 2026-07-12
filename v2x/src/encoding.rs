use c_its_parser::EncodingRules;

/// Supported ASN.1 encoding rules for V2X message payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum V2xEncoding {
    /// Unaligned Packed Encoding Rules.
    Uper,
    /// XML Encoding Rules.
    Xer,
    /// JSON Encoding Rules.
    Jer,
}

impl From<V2xEncoding> for EncodingRules {
    fn from(enc: V2xEncoding) -> EncodingRules {
        match enc {
            V2xEncoding::Uper => EncodingRules::UPER,
            V2xEncoding::Xer => EncodingRules::XER,
            V2xEncoding::Jer => EncodingRules::JER,
        }
    }
}
