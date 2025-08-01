/// An enum to pass messages from the command runners to the webhook sender.
#[derive(Clone, Debug)]
pub enum StreamMessage {
    Line(String),
    Flush,
    CommandFinished,
}
