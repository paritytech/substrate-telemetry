use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Node {
    pub name: Box<str>,
    pub implementation: Box<str>,
    pub version: Box<str>,
}
