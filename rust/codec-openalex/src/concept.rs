#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Concept {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub score: Option<f64>,
}
