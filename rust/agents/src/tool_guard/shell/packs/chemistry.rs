//! Chemistry pack: `chemistry.molecular_dynamics`.

use super::{Confidence, Pack, PatternRule, destructive_pattern};

// ---------------------------------------------------------------------------
// chemistry.molecular_dynamics
// ---------------------------------------------------------------------------

pub static MOLECULAR_DYNAMICS_PACK: Pack = Pack {
    id: "chemistry.molecular_dynamics",
    name: "Molecular Dynamics",
    description: "Guards against destructive molecular dynamics and chemistry tool operations",
    destructive_patterns: &[
        destructive_pattern!(
            "gmx_trjconv_overwrite",
            r"\bgmx\s+trjconv\b",
            "Trajectory conversion can overwrite existing trajectory files",
            "Use `-o` with a new filename; backup the original trajectory first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "gmx_eneconv_overwrite",
            r"\bgmx\s+eneconv\b",
            "Energy file conversion can overwrite existing energy files",
            "Use `-o` with a new filename; backup the original file first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "obabel_overwrite",
            r"\bobabel\b.*(?:--overwrite|-O)\b",
            "Open Babel format conversion overwrites output files",
            "Verify the output path; backup existing files first",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- Molecular Dynamics --

    #[test]
    fn gmx_trjconv_overwrite_matches() {
        let re =
            Regex::new(rule_by_id(&MOLECULAR_DYNAMICS_PACK, "gmx_trjconv_overwrite").pattern)
                .expect("pattern should compile");
        assert!(re.is_match("gmx trjconv -s topol.tpr -f traj.xtc -o output.xtc"));
        assert!(re.is_match("gmx trjconv -f traj.trr -o trimmed.trr -b 100 -e 500"));
        assert!(!re.is_match("gmx mdrun -s topol.tpr"));
        assert!(!re.is_match("gmx grompp -f md.mdp -c conf.gro"));
        assert!(!re.is_match("gmx energy -f ener.edr"));
    }

    #[test]
    fn gmx_eneconv_overwrite_matches() {
        let re =
            Regex::new(rule_by_id(&MOLECULAR_DYNAMICS_PACK, "gmx_eneconv_overwrite").pattern)
                .expect("pattern should compile");
        assert!(re.is_match("gmx eneconv -f ener1.edr ener2.edr -o combined.edr"));
        assert!(re.is_match("gmx eneconv -f ener.edr -o trimmed.edr -b 100"));
        assert!(!re.is_match("gmx energy -f ener.edr"));
        assert!(!re.is_match("gmx mdrun -s topol.tpr"));
        assert!(!re.is_match("gmx trjconv -f traj.xtc"));
    }

    #[test]
    fn obabel_overwrite_matches() {
        let re = Regex::new(rule_by_id(&MOLECULAR_DYNAMICS_PACK, "obabel_overwrite").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("obabel input.sdf --overwrite -O output.mol2"));
        assert!(re.is_match("obabel input.pdb -O output.sdf"));
        assert!(re.is_match("obabel molecule.xyz --overwrite -o mol2"));
        assert!(!re.is_match("obabel input.sdf -o mol2"));
        assert!(!re.is_match("obabel -L formats"));
        assert!(!re.is_match("obabel input.sdf -h"));
    }
}
