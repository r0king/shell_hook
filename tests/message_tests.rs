use hook_stream::message::StreamMessage;

#[test]
fn test_stream_message_clone() {
    let msg1 = StreamMessage::Line("hello".to_string());
    let msg2 = msg1.clone();
    if let StreamMessage::Line(s) = msg2 {
        assert_eq!(s, "hello");
    } else {
        panic!("Cloned message is not a Line variant");
    }

    let msg3 = StreamMessage::CommandFinished;
    let msg4 = msg3.clone();
    assert!(matches!(msg4, StreamMessage::CommandFinished));
}
