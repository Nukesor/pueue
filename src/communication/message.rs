use ::serde_derive::{Deserialize, Serialize};

/// The Message used to add a new command to the daemon.
#[derive(Serialize, Deserialize)]
pub enum Message {
    Add(AddMessage),
    Remove(RemoveMessage),
    Switch(SwitchMessage),

    Start(StartMessage),
    Pause(PauseMessage),
    Kill(KillMessage),

    Reset,
    Clear,

    Status,
    Success,
    Failure,
}

#[derive(Serialize, Deserialize)]
pub struct AddMessage {
    pub command: String,
    pub path: String,
}


#[derive(Serialize, Deserialize)]
pub struct RemoveMessage {
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SwitchMessage {
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct StartMessage {
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct PauseMessage{
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct KillMessage{
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SuccessMessage{
    pub command: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct FailureMessage{
    pub command: String,
    pub path: String,
}
