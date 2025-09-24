#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SummaryStats {
    #[serde(rename = "2yr_mean_citedness")]
    pub impact_factor: Option<f64>,
    pub h_index: Option<i32>,
    pub i10_index: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CountsByYear {
    pub year: Option<i32>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Role {
    pub role: Option<String>,
    pub id: Option<String>,
    pub works_count: Option<i64>,
}
