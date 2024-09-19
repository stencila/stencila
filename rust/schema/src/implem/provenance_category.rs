use ProvenanceCategory::*;

use crate::ProvenanceCategory;

impl ProvenanceCategory {
    pub fn is_human_written(&self) -> bool {
        matches!(
            self,
            HwHeHv | HwHe | HwHv | Hw | HwMv | HwMeHv | HwMe | HwMeMv
        )
    }

    pub fn is_human_edited(&self) -> bool {
        matches!(self, HwHeHv | HwHe | MwHeHv | MwHe | MwHeMv)
    }

    pub fn is_machine_written(&self) -> bool {
        matches!(
            self,
            MwHeHv | MwHe | MwHeMv | MwHv | MwMeHv | Mw | MwMv | MwMe | MwMeMv
        )
    }

    pub fn is_verified(&self) -> bool {
        matches!(
            self,
            HwHeHv
                | HwHv
                | HwMv
                | HwMeHv
                | HwMeMv
                | MwHeHv
                | MwHeMv
                | MwHv
                | MwMeHv
                | MwMv
                | MwMeMv
        )
    }
}
