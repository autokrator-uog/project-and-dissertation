#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub timestamp: String,
    pub from: String,
    pub data: String
}
