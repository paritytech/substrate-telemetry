use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NodeDetails {
    pub name: Box<str>,
    pub implementation: Box<str>,
    pub version: Box<str>,
}

pub struct Node {
    details: NodeDetails,
}

impl Node {
    pub fn new(details: NodeDetails) -> Self {
        Node {
            details,
        }
    }
}
