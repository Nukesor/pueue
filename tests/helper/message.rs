use pueue_lib::network::message::Message;

pub fn assert_success(message: Message) {
    assert!(
        matches!(message, Message::Success(_)),
        "Expected to get SuccessMessage, got {:?}",
        message
    );
}

pub fn assert_failure(message: Message) {
    assert!(
        matches!(message, Message::Failure(_)),
        "Expected to get FailureMessage, got {:?}",
        message
    );
}
