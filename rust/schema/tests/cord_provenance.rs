use schema::cord_provenance::*;

#[test]
fn cord_provenance() {
    use schema::ProvenanceCategory::*;

    let prov = human_written();
    assert_eq!(prov, 0b00000000);
    assert_eq!(display(prov), "Hw");
    assert_eq!(category(prov), Hw);

    let prov = machine_written();
    assert_eq!(prov, 0b00000001);
    assert_eq!(display(prov), "Mw");
    assert_eq!(category(prov), Mw);

    // Adding editors

    let prov = human_edited(prov);
    assert_eq!(prov, 0b00000011);
    assert_eq!(display(prov), "MwHe");
    assert_eq!(category(prov), MwHe);

    let prov = machine_edited(prov);
    assert_eq!(display(prov), "MwMe");
    assert_eq!(category(prov), MwMe);

    let prov = human_edited(prov);
    assert_eq!(display(prov), "MwHe");
    assert_eq!(category(prov), MwHe);

    // Adding verifiers

    let prov = human_verified(prov);
    assert_eq!(display(prov), "MwHeHv");
    assert_eq!(category(prov), MwHeHv);

    let prov = machine_verified(prov);
    assert_eq!(display(prov), "MwHeHvMv");
    assert_eq!(category(prov), MwHeHv);

    let prov = machine_verified(prov);
    assert_eq!(display(prov), "MwHeHv2Mv");
    assert_eq!(category(prov), MwHeHv);

    let prov = machine_verified(prov);
    assert_eq!(display(prov), "MwHeHv3Mv");
    assert_eq!(category(prov), MwHeHv);

    let prov = machine_verified(prov);
    assert_eq!(display(prov), "MwHeHv3Mv"); // Maximum 3Mv
    assert_eq!(category(prov), MwHeHv);

    // Adding editor will clear verifiers

    let prov = machine_edited(prov);
    assert_eq!(display(prov), "MwMe");
    assert_eq!(category(prov), MwMe);

    let prov = machine_verified(prov);
    assert_eq!(display(prov), "MwMeMv");
    assert_eq!(category(prov), MwMeMv);

    let prov = human_edited(prov);
    assert_eq!(display(prov), "MwHe");
    assert_eq!(category(prov), MwHe);

    // Adding verifiers again

    let prov = human_verified(prov);
    assert_eq!(display(prov), "MwHeHv");
    assert_eq!(category(prov), MwHeHv);

    let prov = human_verified(prov);
    assert_eq!(display(prov), "MwHe2Hv");
    assert_eq!(category(prov), MwHeHv);

    let prov = human_verified(prov);
    assert_eq!(display(prov), "MwHe3Hv");
    assert_eq!(category(prov), MwHeHv);
}
