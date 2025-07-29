use common::serde_json;
use common_dev::{pretty_assertions::assert_eq, proptest::prelude::*};

use schema::{
    AuthorType, Block, Cord, CordAuthorship,
    cord_provenance::{display, human_written},
};

#[test]
#[allow(clippy::unwrap_used)]
fn serialization() {
    // Test deserialization of a union type containing a cord with
    // authorship
    serde_json::from_str::<Block>(
        r#"{
            "type": "Paragraph",
            "content": [
                {
                    "type": "Text",
                    "value": {
                        "string": "abc",
                        "authorship": [
                            [
                                1,
                                0,
                                0,
                                3
                            ]
                        ]
                    }
                }
            ],
            "authors": [
                {
                    "type": "AuthorRole",
                    "author": {
                        "type": "Person",
                        "givenNames": [
                            "Alice"
                        ]
                    },
                    "roleName": "Importer"
                }
            ]
        }"#,
    )
    .unwrap();
}

#[test]
#[allow(clippy::unwrap_used)]
fn update_authors() {
    let update = Cord::update_authors;
    let extract = Cord::extract_authors;

    let count = 0;
    let authors = 0;
    let prov = human_written();

    use AuthorType::*;

    let (count, authors, prov) = update(count, authors, prov, 0, Human).unwrap();
    assert_eq!(count, 1);
    assert_eq!(extract(count, authors), vec![0]);
    assert_eq!(display(prov), "HwHe");

    let (count, authors, prov) = update(count, authors, prov, 1, Machine).unwrap();
    assert_eq!(count, 2);
    assert_eq!(extract(count, authors), vec![1, 0]);
    assert_eq!(display(prov), "HwMe");

    let result = update(count, authors, prov, 1, Machine);
    assert!(result.is_none());

    let (count, authors, prov) = update(count, authors, prov, 2, Human).unwrap();
    assert_eq!(count, 3);
    assert_eq!(extract(count, authors), vec![2, 1, 0]);
    assert_eq!(display(prov), "HwHe");

    let (count, authors, prov) = update(count, authors, prov, 3, Human).unwrap();
    assert_eq!(count, 4);
    assert_eq!(extract(count, authors), vec![3, 2, 1, 0]);
    assert_eq!(display(prov), "HwHe");

    let (count, authors, prov) = update(count, authors, prov, 4, Machine).unwrap();
    assert_eq!(count, 5);
    assert_eq!(extract(count, authors), vec![4, 3, 2, 1, 0]);
    assert_eq!(display(prov), "HwMe");

    let (count, authors, prov) = update(count, authors, prov, 5, Human).unwrap();
    assert_eq!(count, 6);
    assert_eq!(extract(count, authors), vec![5, 4, 3, 2, 1, 0]);
    assert_eq!(display(prov), "HwHe");

    let (count, authors, prov) = update(count, authors, prov, 6, Machine).unwrap();
    assert_eq!(count, 7);
    assert_eq!(extract(count, authors), vec![6, 5, 4, 3, 2, 1, 0]);
    assert_eq!(display(prov), "HwMe");

    let (count, authors, prov) = update(count, authors, prov, 7, Human).unwrap();
    assert_eq!(count, 8);
    assert_eq!(extract(count, authors), vec![7, 6, 5, 4, 3, 2, 1, 0]);
    assert_eq!(display(prov), "HwHe");

    let (count, authors, prov) = update(count, authors, prov, 8, Human).unwrap();
    assert_eq!(count, 9);
    assert_eq!(extract(count, authors), vec![8, 7, 6, 5, 4, 3, 2, 1]);
    assert_eq!(display(prov), "HwHe");
}

/// Create a `CordRun` that has a default provenance
fn run(count: u8, authors: u64, length: u32) -> CordAuthorship {
    CordAuthorship::new(count, authors, human_written(), length)
}

#[test]
fn insert_at_start() {
    let mut cord = Cord {
        string: "world!".to_string(),
        authorship: vec![run(1, 0, 6)],
    };
    cord.apply_insert(0, "Hello, ", Some(1), None);
    assert_eq!(cord.string, "Hello, world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 7), run(1, 0, 6)]);
}

#[test]
fn insert_at_end() {
    let mut cord = Cord {
        string: "Hello".to_string(),
        authorship: vec![run(1, 1, 5)],
    };
    cord.apply_insert(5, ", world!", Some(1), None);
    assert_eq!(cord.string, "Hello, world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 13)]);
}

#[test]
fn insert_at_middle() {
    let mut cord = Cord {
        string: "Hello world!".to_string(),
        authorship: vec![run(1, 1, 5), run(1, 2, 7)],
    };
    cord.apply_insert(5, ", beautiful", Some(3), None);
    assert_eq!(cord.string, "Hello, beautiful world!");
    assert_eq!(
        cord.authorship,
        vec![run(1, 1, 5), run(1, 3, 11), run(1, 2, 7)]
    );
}

#[test]
fn insert_nothing() {
    let mut cord = Cord {
        string: "Hello".to_string(),
        authorship: vec![run(1, 1, 5)],
    };
    cord.apply_insert(3, "", Some(1), None); // Empty string insertSome(i)on
    assert_eq!(cord.string, "Hello");
    assert_eq!(cord.authorship, vec![run(1, 1, 5)]);
}

#[test]
fn insert_out_of_bounds() {
    let mut cord = Cord {
        string: "Hello".to_string(),
        authorship: vec![run(1, 1, 5)],
    };
    cord.apply_insert(10, " world", Some(1), None); // Beyond the length of the strSome(i)ng
    assert_eq!(cord.string, "Hello");
    assert_eq!(cord.authorship, vec![run(1, 1, 5)]); // No change
}

#[test]
fn delete_entire_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 7), run(1, 2, 6)],
    };
    cord.apply_delete(0..7);
    assert_eq!(cord.string, "world!");
    assert_eq!(cord.authorship, vec![run(1, 2, 6)]);
}

#[test]
fn delete_within_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 13)],
    };
    cord.apply_delete(0..6);
    assert_eq!(cord.string, " world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 7)]);

    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 13)],
    };
    cord.apply_delete(5..13);
    assert_eq!(cord.string, "Hello");
    assert_eq!(cord.authorship, vec![run(1, 1, 5)]);

    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 13)],
    };
    cord.apply_delete(1..12);
    assert_eq!(cord.string, "H!");
    assert_eq!(cord.authorship, vec![run(1, 1, 2)]);
}

#[test]
fn delete_across_runs() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 0, 6), run(1, 1, 7)],
    };
    cord.apply_delete(5..12);
    assert_eq!(cord.string, "Hello!");
    assert_eq!(cord.authorship, vec![run(1, 0, 5), run(1, 1, 1)]);

    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 3), run(1, 2, 2), run(1, 3, 8)],
    };
    cord.apply_delete(1..12);
    assert_eq!(cord.string, "H!");
    assert_eq!(cord.authorship, vec![run(1, 1, 1), run(1, 3, 1)]);
}

#[test]
fn delete_at_edges() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 7), run(1, 2, 6)],
    };

    cord.apply_delete(0..5); // Beginning edge
    assert_eq!(cord.string, ", world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 2), run(1, 2, 6)]);

    cord.apply_delete(5..8); // End edge
    assert_eq!(cord.string, ", wor");
    assert_eq!(cord.authorship, vec![run(1, 1, 2), run(1, 2, 3)]);
}

#[test]
fn delete_no_effect() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 7), run(1, 2, 6)],
    };
    cord.apply_delete(14..20); // Beyond string length
    assert_eq!(cord.string, "Hello, world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 7), run(1, 2, 6)]);
}

#[test]
fn delete_empty_range() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 7), run(1, 2, 6)],
    };
    cord.apply_delete(5..5); // Empty range should do nothing
    assert_eq!(cord.string, "Hello, world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 7), run(1, 2, 6)]);
}

#[test]
fn delete_from_empty() {
    let mut cord = Cord {
        string: "".to_string(),
        authorship: Vec::new(),
    };
    cord.apply_delete(0..1); // Deleting from an empty string
    assert_eq!(cord.string, "");
    assert_eq!(cord.authorship, Vec::new());
}

#[test]
fn replace_entire_run() {
    let mut cord = Cord {
        string: "a".to_string(),
        authorship: vec![run(1, 1, 1)],
    };
    cord.apply_replace(0..1, " b", Some(1), None);

    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 5), run(1, 2, 8)],
    };

    cord.apply_replace(0..5, "Howdy", Some(1), None); // Same author
    assert_eq!(cord.string, "Howdy, world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 5), run(1, 2, 8)]);

    cord.apply_replace(0..5, "Heyya", Some(3), None); // Different author
    assert_eq!(cord.string, "Heyya, world!");
    assert_eq!(cord.runs(), 2);
    assert_eq!(cord.run_authors(0), vec![3, 1]);
    assert_eq!(cord.run_length(0), 5);
    assert_eq!(cord.authorship[1], run(1, 2, 8));
}

#[test]
fn replace_start_of_a_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 13)],
    };

    cord.apply_replace(0..5, "Hi", Some(1), None); // Same author
    assert_eq!(cord.string, "Hi, world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 10)]);

    cord.apply_replace(0..5, "Yo! W", Some(2), None); // Different author
    assert_eq!(cord.string, "Yo! World!");
    assert_eq!(cord.runs(), 2);
    assert_eq!(cord.run_authors(0), vec![2, 1]);
    assert_eq!(cord.run_length(0), 5);
    assert_eq!(cord.authorship[1], run(1, 1, 5));

    cord.apply_replace(0..1, "Hey, y", Some(3), None); // Another author
    assert_eq!(cord.string, "Hey, yo! World!");
    assert_eq!(cord.runs(), 3);
    assert_eq!(cord.run_authors(0), vec![3, 2, 1]);
    assert_eq!(cord.run_length(0), 6);
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 4);
    assert_eq!(cord.authorship[2], run(1, 1, 5));
}

#[test]
fn replace_end_of_a_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 13)],
    };

    cord.apply_replace(7..13, "their", Some(1), None); // Same author
    assert_eq!(cord.string, "Hello, their");
    assert_eq!(cord.authorship, vec![run(1, 1, 12)]);

    cord.apply_replace(10..12, "re.", Some(2), None); // Different author
    assert_eq!(cord.string, "Hello, there.");
    assert_eq!(cord.runs(), 2);
    assert_eq!(cord.authorship[0], run(1, 1, 10));
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 3);

    cord.apply_replace(12..13, "!", Some(3), None); // Another author
    assert_eq!(cord.string, "Hello, there!");
    assert_eq!(cord.runs(), 3);
    assert_eq!(cord.authorship[0], run(1, 1, 10));
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 2);
    assert_eq!(cord.run_authors(2), vec![3, 2, 1]);
    assert_eq!(cord.run_length(2), 1);
}

#[test]
fn replace_within_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        authorship: vec![run(1, 1, 13)],
    };

    cord.apply_replace(1..5, "ey", Some(1), None); // Same author
    assert_eq!(cord.string, "Hey, world!");
    assert_eq!(cord.authorship, vec![run(1, 1, 11)]);

    cord.apply_replace(1..3, "owdy", Some(2), None); // Different author
    assert_eq!(cord.string, "Howdy, world!");
    assert_eq!(cord.runs(), 3);
    assert_eq!(cord.authorship[0], run(1, 1, 1));
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 4);
    assert_eq!(cord.authorship[2], run(1, 1, 8));

    cord.apply_replace(4..5, "'y", Some(3), None); // Another author
    assert_eq!(cord.string, "Howd'y, world!");
    assert_eq!(cord.runs(), 4);
    assert_eq!(cord.authorship[0], run(1, 1, 1));
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 3);
    assert_eq!(cord.run_authors(2), vec![3, 2, 1]);
    assert_eq!(cord.run_length(2), 2);
    assert_eq!(cord.authorship[3], run(1, 1, 8));
}

#[test]
fn replace_across_runs() {
    let cord = Cord {
        string: "aaaabbbbccccdddd".to_string(),
        authorship: vec![run(1, 1, 4), run(1, 2, 4), run(1, 3, 4), run(1, 4, 4)],
    };

    // First author at start, equal replacement
    let mut c = cord.clone();
    c.apply_replace(0..6, "------", Some(1), None);
    assert_eq!(c.string, "------bbccccdddd");
    assert_eq!(c.authorship[0], run(1, 1, 6));
    assert_eq!(c.authorship[1], run(1, 2, 2));
    assert_eq!(c.authorship[2], run(1, 3, 4));
    assert_eq!(c.authorship[3], run(1, 4, 4));

    // First author at start, shorter, replacement
    let mut c = cord.clone();
    c.apply_replace(0..6, "----", Some(1), None);
    assert_eq!(c.string, "----bbccccdddd");
    assert_eq!(c.authorship[0], run(1, 1, 4));
    assert_eq!(c.authorship[1], run(1, 2, 2));
    assert_eq!(c.authorship[2], run(1, 3, 4));
    assert_eq!(c.authorship[3], run(1, 4, 4));

    // First author at start, longer replacement
    let mut c = cord.clone();
    c.apply_replace(0..6, "--------", Some(1), None);
    assert_eq!(c.string, "--------bbccccdddd");
    assert_eq!(c.authorship[0], run(1, 1, 8));
    assert_eq!(c.authorship[1], run(1, 2, 2));
    assert_eq!(c.authorship[2], run(1, 3, 4));
    assert_eq!(c.authorship[3], run(1, 4, 4));

    // New author, shorter replacement in middle
    let mut c = cord.clone();
    c.apply_replace(6..10, "--", Some(5), None);
    assert_eq!(c.string, "aaaabb--ccdddd");
    assert_eq!(c.authorship[0], run(1, 1, 4));
    assert_eq!(c.authorship[1], run(1, 2, 2));
    assert_eq!(c.run_authors(2), vec![5, 2]);
    assert_eq!(c.run_length(2), 2);
    assert_eq!(c.authorship[3], run(1, 3, 2));
    assert_eq!(c.authorship[4], run(1, 4, 4));

    // New author, wide, longer replacement in middle
    let mut c = cord.clone();
    c.apply_replace(1..15, "---------------", Some(5), None);
    assert_eq!(c.string, "a---------------d");
    assert_eq!(c.authorship[0], run(1, 1, 1));
    assert_eq!(c.run_authors(1), vec![5, 1]);
    assert_eq!(c.run_length(1), 3);
    assert_eq!(c.run_authors(2), vec![5, 2]);
    assert_eq!(c.run_length(2), 4);
    assert_eq!(c.run_authors(3), vec![5, 3]);
    assert_eq!(c.run_length(3), 4);
    assert_eq!(c.authorship[4], run(1, 5, 4)); // Note additional 4 chars here with only new author
    assert_eq!(c.authorship[5], run(1, 4, 1));

    // New author, as above but ending on boundary of existing run
    let mut c = cord.clone();
    c.apply_replace(1..12, "---------------", Some(5), None);
    assert_eq!(c.string, "a---------------dddd");
    assert_eq!(c.authorship[0], run(1, 1, 1));
    assert_eq!(c.run_authors(1), vec![5, 1]);
    assert_eq!(c.run_length(1), 3);
    assert_eq!(c.run_authors(2), vec![5, 2]);
    assert_eq!(c.run_length(2), 4);
    assert_eq!(c.run_authors(3), vec![5, 3]);
    assert_eq!(c.run_length(3), 4);
    assert_eq!(c.authorship[4], run(1, 5, 4));
    assert_eq!(c.authorship[5], run(1, 4, 4));
}

// Merge two cords. Used for testing that merged value is correct
// and that does not panic due to invalid slots
fn merge_cords(s1: &str, s2: &str, s3: Option<&str>) {
    let mut cord = Cord {
        string: s1.to_string(),
        authorship: vec![run(1, 0, s1.chars().count() as u32)],
    };

    let ops = cord.create_ops(&Cord::from(s2));
    cord.apply_ops(ops, Some(1), None);
    assert_eq!(cord.string, s2);

    let Some(s3) = s3 else { return };

    let ops = cord.create_ops(&Cord::from(s3));
    cord.apply_ops(ops, Some(2), None);
    assert_eq!(cord.string, s3);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn proptest_alpha_num(s1 in "[a-zA-Z0-9]*", s2 in "[a-zA-Z0-9]*") {
        merge_cords(&s1, &s2, None);
    }

    #[test]
    fn proptest_unicode(s1 in "\\PC*", s2 in "\\PC*") {
       merge_cords(&s1, &s2, None);
    }

    /*
    TODO: Enable this test
    #[test]
    fn proptest_unicode_twice(s1 in "\\PC*", s2 in "\\PC*", s3 in "\\PC*") {
       merge_cords(&s1, &s2, Some(&s3));
    }
    */
}

// The following are regression tests for problems found from proptests
// and elsewhere

#[test]
fn no_zero_run_lengths() {
    merge_cords("", "A", None);

    //merge_cords("a", "bba", Some("c"));

    let mut c = Cord {
        string: "and".to_string(),
        authorship: vec![run(1, 0, 1), run(1, 2, 1), run(1, 1, 1)],
    };
    c.apply_replace(2..3, "t", Some(1), None);
    assert_eq!(c.string, "ant");
}

#[test]
fn unicode_merges() {
    merge_cords("", "🌀", None);
    merge_cords("🌀", "", None);
    merge_cords("🌀", "a", None);
}
