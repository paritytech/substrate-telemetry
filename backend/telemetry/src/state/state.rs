use super::chain::Chain;
use std::collections::HashMap;

pub struct State {
    chains: HashMap<Box<str>, Chain>
}

impl State {
    pub fn new() -> State {
        State {
            chains: HashMap::new()
        }
    }
}