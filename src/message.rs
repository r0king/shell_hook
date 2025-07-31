/// An enum to pass messages from the command runners to the webhook sender.
#[derive(Clone)]
pub enum StreamMessage {
    Line(String),
    CommandFinished,
}