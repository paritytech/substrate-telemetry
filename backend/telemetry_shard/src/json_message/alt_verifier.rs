use super::hash::Hash;
use serde::Deserialize;
use common::node_message as internal;

pub use internal::ChainType;

/// The Details info for a alt-verifier node.
#[derive(Deserialize, Debug, Clone)]
pub struct VerifierNodeDetails {
    /// The layer1 chain 's genesis.
    pub layer1_genesis_hash: Hash,
    /// The layer2(producer) chain 's genesis.
    pub layer2_genesis_hash: Hash,
    /// The app id of the layer2 in layer1.
    pub layer2_app_id: u32,
    /// The verifier public key.
    pub verifier: Box<str>,
}

impl From<VerifierNodeDetails> for internal::VerifierNodeDetails {
    fn from(msg: VerifierNodeDetails) -> Self {
        internal::VerifierNodeDetails {
            layer1_genesis_hash: msg.layer1_genesis_hash.into(),
            layer2_genesis_hash: msg.layer2_genesis_hash.into(),
            layer2_app_id: msg.layer2_app_id,
            verifier: msg.verifier,
        }
    }
}

/// The Details info for a alt-verifier node.
#[derive(Deserialize, Debug, Clone)]
pub struct VerifierProcessFinalityBlock {
    pub number: u64,
    pub hash: Hash,
    pub expect_number: u64,
}

impl From<VerifierProcessFinalityBlock> for internal::VerifierProcessFinalityBlock {
    fn from(msg: VerifierProcessFinalityBlock) -> Self {
        internal::VerifierProcessFinalityBlock {
            number: msg.number,
            hash: msg.hash.into(),
            expect_number: msg.expect_number,
        }
    }
}
