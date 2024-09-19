use crate::ProvenanceCount;

impl ProvenanceCount {
    /// Calculate the aggregate percentage of human written or edited content
    pub fn human_percent(counts: &[Self]) -> u64 {
        let mut human = 0;
        let mut all = 0;
        for count in counts {
            if count.provenance_category.is_human_written()
                || count.provenance_category.is_human_edited()
            {
                human += count.character_count;
            }
            all += count.character_count;
        }

        (human as f64 / all as f64 * 100.).round().min(100.) as u64
    }

    /// Calculate the aggregate percentage of human written or machine verified content
    pub fn verified_percent(counts: &[Self]) -> u64 {
        let mut verified = 0;
        let mut all = 0;
        for count in counts {
            if count.provenance_category.is_verified() {
                verified += count.character_count;
            }
            all += count.character_count;
        }

        (verified as f64 / all as f64 * 100.).round().min(100.) as u64
    }
}
