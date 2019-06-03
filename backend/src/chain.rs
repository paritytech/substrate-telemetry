use actix::prelude::*;

pub struct Chain;

impl Actor for Chain {
    type Context = Context<Self>;
}
