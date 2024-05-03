use schema::cord_mi::*;

#[test]
fn cord_mi() {
    use Mic::*;

    let mi = human_written();
    assert_eq!(mi, 0b00000000);
    assert_eq!(display(mi), "Hw");
    assert_eq!(category(mi), Hw);

    let mi = machine_written();
    assert_eq!(mi, 0b00000001);
    assert_eq!(display(mi), "Mw");
    assert_eq!(category(mi), Mw);

    // Adding editors

    let mi = human_edited(mi);
    assert_eq!(mi, 0b00000011);
    assert_eq!(display(mi), "MwHe");
    assert_eq!(category(mi), MwHe);

    let mi = machine_edited(mi);
    assert_eq!(display(mi), "MwMe");
    assert_eq!(category(mi), MwMe);

    let mi = human_edited(mi);
    assert_eq!(display(mi), "MwHe");
    assert_eq!(category(mi), MwHe);

    // Adding verifiers

    let mi = human_verified(mi);
    assert_eq!(display(mi), "MwHeHv");
    assert_eq!(category(mi), MwHeHv);

    let mi = machine_verified(mi);
    assert_eq!(display(mi), "MwHeMvHv");
    assert_eq!(category(mi), MwHeHv);

    let mi = machine_verified(mi);
    assert_eq!(display(mi), "MwHeMvMvHv");
    assert_eq!(category(mi), MwHeHv);

    let mi = machine_verified(mi);
    assert_eq!(display(mi), "MwHeMvMvMv");
    assert_eq!(category(mi), MwHeMv);

    // Adding editor will clear verifiers

    let mi = machine_edited(mi);
    assert_eq!(display(mi), "MwMe");
    assert_eq!(category(mi), MwMe);

    let mi = machine_verified(mi);
    assert_eq!(display(mi), "MwMeMv");
    assert_eq!(category(mi), MwMeMv);

    let mi = human_edited(mi);
    assert_eq!(display(mi), "MwHe");
    assert_eq!(category(mi), MwHe);
}
