use schema::cord_mi::*;

#[test]
fn cord_mi() {
    let mi = human_written();
    assert_eq!(mi, 0b00000000);
    assert_eq!(display(mi), "H");

    let mi = machine_written();
    assert_eq!(mi, 0b00000001);
    assert_eq!(display(mi), "M");

    // Adding editors

    let mi = human_edited(mi);
    assert_eq!(mi, 0b00000011);
    assert_eq!(display(mi), "MH");

    let mi = machine_edited(mi);
    assert_eq!(display(mi), "MM");

    let mi = human_edited(mi);
    assert_eq!(display(mi), "MH");

    // Adding verifiers

    let mi = human_verified(mi);
    assert_eq!(display(mi), "MH-H");

    let mi = machine_verified(mi);
    assert_eq!(display(mi), "MH-MH");

    let mi = human_verified(mi);
    assert_eq!(display(mi), "MH-HMH");

    let mi = human_verified(mi);
    assert_eq!(display(mi), "MH-HHM");

    // Adding editor will clear verifiers

    let mi = machine_edited(mi);
    assert_eq!(display(mi), "MM");

    let mi = machine_verified(mi);
    assert_eq!(display(mi), "MM-M");

    let mi = human_edited(mi);
    assert_eq!(display(mi), "MH");
}
