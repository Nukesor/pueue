#[derive(Serialize, Deserialize)]
pub struct AddMessage {
    command: String,
    path: String,
}
