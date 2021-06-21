//! The structs and enums defined in this module are largely identical to those
//! we'll use elsewhere internally, but are kept separate so that the JSON structure
//! is defined (almost) from just this file, and we don't have to worry about breaking
//! compatibility with the input data when we make changes to our internal data
//! structures (for example, to support bincode better).
use super::hash::Hash;
use serde::{Deserialize};

/// This struct represents a telemetry message sent from a node as
/// a JSON payload. Since JSON is self describing, we can use attributes
/// like serde(untagged) and serde(flatten) without issue.
///
/// Internally, we want to minimise the amount of data sent from shards to
/// the core node. For that reason, we use a non-self-describing serialization
/// format like bincode, which doesn't support things like `[serde(flatten)]` (which
/// internally wants to serialize to a map of unknown length) or `[serde(tag/untagged)]`
/// (which relies on the data to know which variant to deserialize to.)
///
/// So, this can be converted fairly cheaply into an enum we'll use internally
/// which is compatible with formats like bincode.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum NodeMessage {
    V1 {
        #[serde(flatten)]
        payload: Payload,
    },
    V2 {
        id: NodeMessageId,
        payload: Payload,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "msg")]
pub enum Payload {
    #[serde(rename = "system.connected")]
    SystemConnected(SystemConnected),
    #[serde(rename = "system.interval")]
    SystemInterval(SystemInterval),
    #[serde(rename = "block.import")]
    BlockImport(Block),
    #[serde(rename = "notify.finalized")]
    NotifyFinalized(Finalized),
    #[serde(rename = "txpool.import")]
    TxPoolImport,
    #[serde(rename = "afg.finalized")]
    AfgFinalized(AfgFinalized),
    #[serde(rename = "afg.received_precommit")]
    AfgReceivedPrecommit(AfgReceived),
    #[serde(rename = "afg.received_prevote")]
    AfgReceivedPrevote(AfgReceived),
    #[serde(rename = "afg.received_commit")]
    AfgReceivedCommit(AfgReceived),
    #[serde(rename = "afg.authority_set")]
    AfgAuthoritySet(AfgAuthoritySet),
    #[serde(rename = "afg.finalized_blocks_up_to")]
    AfgFinalizedBlocksUpTo,
    #[serde(rename = "aura.pre_sealed_block")]
    AuraPreSealedBlock,
    #[serde(rename = "prepared_block_for_proposing")]
    PreparedBlockForProposing,
}

#[derive(Deserialize, Debug)]
pub struct SystemConnected {
    pub genesis_hash: Hash,
    #[serde(flatten)]
    pub node: NodeDetails,
}

#[derive(Deserialize, Debug)]
pub struct SystemInterval {
    pub peers: Option<u64>,
    pub txcount: Option<u64>,
    pub bandwidth_upload: Option<f64>,
    pub bandwidth_download: Option<f64>,
    pub finalized_height: Option<BlockNumber>,
    pub finalized_hash: Option<Hash>,
    #[serde(flatten)]
    pub block: Option<Block>,
    pub used_state_cache_size: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct Finalized {
    #[serde(rename = "best")]
    pub hash: Hash,
    pub height: Box<str>,
}

#[derive(Deserialize, Debug)]
pub struct AfgAuthoritySet {
    pub authority_id: Box<str>,
    pub authorities: Box<str>,
    pub authority_set_id: Box<str>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AfgFinalized {
    pub finalized_hash: Hash,
    pub finalized_number: Box<str>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AfgReceived {
    pub target_hash: Hash,
    pub target_number: Box<str>,
    pub voter: Option<Box<str>>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Block {
    #[serde(rename = "best")]
    pub hash: Hash,
    pub height: BlockNumber,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeDetails {
    pub chain: Box<str>,
    pub name: Box<str>,
    pub implementation: Box<str>,
    pub version: Box<str>,
    pub validator: Option<Box<str>>,
    pub network_id: Option<Box<str>>,
    pub startup_time: Option<Box<str>>,
}

type NodeMessageId = u64;
type BlockNumber = u64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_v1() {
        let json = r#"{
            "msg":"notify.finalized",
            "level":"INFO",
            "ts":"2021-01-13T12:38:25.410794650+01:00",
            "best":"0x031c3521ca2f9c673812d692fc330b9a18e18a2781e3f9976992f861fd3ea0cb",
            "height":"50"
        }"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V1 { .. },
            ),
            "message did not match variant V1",
        );
    }

    #[test]
    fn message_v2() {
        let json = r#"{
            "id":1,
            "ts":"2021-01-13T12:22:20.053527101+01:00",
            "payload":{
                "best":"0xcc41708573f2acaded9dd75e07dac2d4163d136ca35b3061c558d7a35a09dd8d",
                "height":"209",
                "msg":"notify.finalized"
            }
        }"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V2 { .. },
            ),
            "message did not match variant V2",
        );
    }

    #[test]
    fn message_v2_received_precommit() {
        let json = r#"{
            "id":1,
            "ts":"2021-01-13T12:22:20.053527101+01:00",
            "payload":{
                "target_hash":"0xcc41708573f2acaded9dd75e07dac2d4163d136ca35b3061c558d7a35a09dd8d",
                "target_number":"209",
                "voter":"foo",
                "msg":"afg.received_precommit"
            }
        }"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V2 {
                    payload: Payload::AfgReceivedPrecommit(..),
                    ..
                },
            ),
            "message did not match the expected output",
        );
    }

    #[test]
    fn message_v2_tx_pool_import() {
        // We should happily ignore any fields we don't care about.
        let json = r#"{
            "id":1,
            "ts":"2021-01-13T12:22:20.053527101+01:00",
            "payload":{
                "foo":"Something",
                "bar":123,
                "wibble":"wobble",
                "msg":"txpool.import"
            }
        }"#;
        assert!(
            matches!(
                serde_json::from_str::<NodeMessage>(json).unwrap(),
                NodeMessage::V2 {
                    payload: Payload::TxPoolImport,
                    ..
                },
            ),
            "message did not match the expected output",
        );
    }
}
