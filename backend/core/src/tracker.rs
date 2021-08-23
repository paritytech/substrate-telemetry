use actix::prelude::*;
use actix::Handler;
use sqlx::PgPool;

pub struct Tracker {
    pool: Option<PgPool>,
}

/// Message sent from the Connector to the Tracker on heartbeat
#[derive(Message)]
#[rtype(result = "sqlx::Result<()>")]
pub struct Heartbeat(pub String);

impl Tracker {
    pub fn new(pool: Option<PgPool>) -> Self {
        Self { pool }
    }
}

impl Actor for Tracker {
    type Context = Context<Self>;
}

impl Handler<Heartbeat> for Tracker {
    type Result = ResponseFuture<sqlx::Result<()>>;

    fn handle(&mut self, msg: Heartbeat, _ctx: &mut Context<Self>) -> Self::Result {
        let pool = self.pool.clone();
        Box::pin(async move {
            if let Some(db_pool) = pool {
                sqlx::query(
                    r#"
    INSERT INTO heartbeats (access_key)
    VALUES ( $1 )
                "#,
                )
                .bind(msg.0)
                .execute(&db_pool)
                .await
                .map(|_| ())?
            }
            Ok(())
        })
    }
}
