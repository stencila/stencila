pub type Subscriber = fn(topic: String, data: serde_json::Value) -> ();

pub fn publish(_topic: &str, _event: &serde_json::Value) {}
