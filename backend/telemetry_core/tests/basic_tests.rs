use test_utils::{feed_message_de::FeedMessage, server::Server};
// use serde_json::json;

#[tokio::test]
async fn can_ping_feed() {

    let server = Server::start_default()
        .await
        .expect("server could start");

    // Connect to the feed:
    let (mut feed_tx, mut feed_rx) = server.get_core().connect().await.unwrap();

    // Expect a version response of 31:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_eq!(feed_messages, vec![FeedMessage::Version(31)], "expecting version");

    // Ping it:
    feed_tx.send_command("ping", "hello!").await.unwrap();

    // Expect a pong response:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_eq!(feed_messages, vec![FeedMessage::Pong { msg: "hello!".to_owned() }], "expecting pong");

    // Tidy up:
    server.shutdown().await;
}