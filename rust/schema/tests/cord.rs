use common::serde_json;
use common_dev::{pretty_assertions::assert_eq, proptest::prelude::*};

use schema::{Block, Cord};

#[test]
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
fn update_authors() {
    let update = Cord::update_authors;
    let extract = Cord::extract_authors;

    let count = 0;
    let authors = 0;

    let (count, authors) = update(count, authors, 0).unwrap();
    assert_eq!(count, 1);
    assert_eq!(extract(count, authors), vec![0]);

    let (count, authors) = update(count, authors, 1).unwrap();
    assert_eq!(count, 2);
    assert_eq!(extract(count, authors), vec![1, 0]);

    let result = update(count, authors, 1);
    assert!(result.is_none());

    let (count, authors) = update(count, authors, 2).unwrap();
    assert_eq!(count, 3);
    assert_eq!(extract(count, authors), vec![2, 1, 0]);

    let (count, authors) = update(count, authors, 3).unwrap();
    assert_eq!(count, 4);
    assert_eq!(extract(count, authors), vec![3, 2, 1, 0]);

    let (count, authors) = update(count, authors, 4).unwrap();
    assert_eq!(count, 5);
    assert_eq!(extract(count, authors), vec![4, 3, 2, 1, 0]);

    let (count, authors) = update(count, authors, 5).unwrap();
    assert_eq!(count, 6);
    assert_eq!(extract(count, authors), vec![5, 4, 3, 2, 1, 0]);

    let (count, authors) = update(count, authors, 6).unwrap();
    assert_eq!(count, 7);
    assert_eq!(extract(count, authors), vec![6, 5, 4, 3, 2, 1, 0]);

    let (count, authors) = update(count, authors, 7).unwrap();
    assert_eq!(count, 8);
    assert_eq!(extract(count, authors), vec![7, 6, 5, 4, 3, 2, 1, 0]);

    let (count, authors) = update(count, authors, 8).unwrap();
    assert_eq!(count, 9);
    assert_eq!(extract(count, authors), vec![8, 7, 6, 5, 4, 3, 2, 1]);
}

#[test]
fn insert_at_start() {
    let mut cord = Cord {
        string: "world!".to_string(),
        runs: vec![(1, 0, 6)],
    };
    cord.apply_insert(0, "Hello, ", 1);
    assert_eq!(cord.string, "Hello, world!");
    assert_eq!(cord.runs, vec![(1, 1, 7), (1, 0, 6)]);
}

#[test]
fn insert_at_end() {
    let mut cord = Cord {
        string: "Hello".to_string(),
        runs: vec![(1, 1, 5)],
    };
    cord.apply_insert(5, ", world!", 1);
    assert_eq!(cord.string, "Hello, world!");
    assert_eq!(cord.runs, vec![(1, 1, 13)]);
}

#[test]
fn insert_at_middle() {
    let mut cord = Cord {
        string: "Hello world!".to_string(),
        runs: vec![(1, 1, 5), (1, 2, 7)],
    };
    cord.apply_insert(5, ", beautiful", 3);
    assert_eq!(cord.string, "Hello, beautiful world!");
    assert_eq!(cord.runs, vec![(1, 1, 5), (1, 3, 11), (1, 2, 7)]);
}

#[test]
fn insert_nothing() {
    let mut cord = Cord {
        string: "Hello".to_string(),
        runs: vec![(1, 1, 5)],
    };
    cord.apply_insert(3, "", 1); // Empty string insertion
    assert_eq!(cord.string, "Hello");
    assert_eq!(cord.runs, vec![(1, 1, 5)]);
}

#[test]
fn insert_out_of_bounds() {
    let mut cord = Cord {
        string: "Hello".to_string(),
        runs: vec![(1, 1, 5)],
    };
    cord.apply_insert(10, " world", 1); // Beyond the length of the string
    assert_eq!(cord.string, "Hello");
    assert_eq!(cord.runs, vec![(1, 1, 5)]); // No change
}

#[test]
fn delete_entire_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 7), (1, 2, 6)],
    };
    cord.apply_delete(0..7);
    assert_eq!(cord.string, "world!");
    assert_eq!(cord.runs, vec![(1, 2, 6)]);
}

#[test]
fn delete_within_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 13)],
    };
    cord.apply_delete(0..6);
    assert_eq!(cord.string, " world!");
    assert_eq!(cord.runs, vec![(1, 1, 7)]);

    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 13)],
    };
    cord.apply_delete(5..13);
    assert_eq!(cord.string, "Hello");
    assert_eq!(cord.runs, vec![(1, 1, 5)]);

    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 13)],
    };
    cord.apply_delete(1..12);
    assert_eq!(cord.string, "H!");
    assert_eq!(cord.runs, vec![(1, 1, 2)]);
}

#[test]
fn delete_across_runs() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 0, 6), (1, 1, 7)],
    };
    cord.apply_delete(5..12);
    assert_eq!(cord.string, "Hello!");
    assert_eq!(cord.runs, vec![(1, 0, 5), (1, 1, 1)]);

    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 3), (1, 2, 2), (1, 3, 8)],
    };
    cord.apply_delete(1..12);
    assert_eq!(cord.string, "H!");
    assert_eq!(cord.runs, vec![(1, 1, 1), (1, 3, 1)]);
}

#[test]
fn delete_at_edges() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 7), (1, 2, 6)],
    };

    cord.apply_delete(0..5); // Beginning edge
    assert_eq!(cord.string, ", world!");
    assert_eq!(cord.runs, vec![(1, 1, 2), (1, 2, 6)]);

    cord.apply_delete(5..8); // End edge
    assert_eq!(cord.string, ", wor");
    assert_eq!(cord.runs, vec![(1, 1, 2), (1, 2, 3)]);
}

#[test]
fn delete_no_effect() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 7), (1, 2, 6)],
    };
    cord.apply_delete(14..20); // Beyond string length
    assert_eq!(cord.string, "Hello, world!");
    assert_eq!(cord.runs, vec![(1, 1, 7), (1, 2, 6)]);
}

#[test]
fn delete_empty_range() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 7), (1, 2, 6)],
    };
    cord.apply_delete(5..5); // Empty range should do nothing
    assert_eq!(cord.string, "Hello, world!");
    assert_eq!(cord.runs, vec![(1, 1, 7), (1, 2, 6)]);
}

#[test]
fn delete_from_empty() {
    let mut cord = Cord {
        string: "".to_string(),
        runs: Vec::new(),
    };
    cord.apply_delete(0..1); // Deleting from an empty string
    assert_eq!(cord.string, "");
    assert_eq!(cord.runs, Vec::new());
}

#[test]
fn replace_entire_run() {
    let mut cord = Cord {
        string: "a".to_string(),
        runs: vec![(1, 1, 1)],
    };
    cord.apply_replace(0..1, " b", 1);

    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 5), (1, 2, 8)],
    };

    cord.apply_replace(0..5, "Howdy", 1); // Same author
    assert_eq!(cord.string, "Howdy, world!");
    assert_eq!(cord.runs, vec![(1, 1, 5), (1, 2, 8)]);

    cord.apply_replace(0..5, "Heyya", 3); // Different author
    assert_eq!(cord.string, "Heyya, world!");
    assert_eq!(cord.runs(), 2);
    assert_eq!(cord.run_authors(0), vec![3, 1]);
    assert_eq!(cord.run_length(0), 5);
    assert_eq!(cord.runs[1], (1, 2, 8));
}

#[test]
fn replace_start_of_a_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 13)],
    };

    cord.apply_replace(0..5, "Hi", 1); // Same author
    assert_eq!(cord.string, "Hi, world!");
    assert_eq!(cord.runs, vec![(1, 1, 10)]);

    cord.apply_replace(0..5, "Yo! W", 2); // Different author
    assert_eq!(cord.string, "Yo! World!");
    assert_eq!(cord.runs(), 2);
    assert_eq!(cord.run_authors(0), vec![2, 1]);
    assert_eq!(cord.run_length(0), 5);
    assert_eq!(cord.runs[1], (1, 1, 5));

    cord.apply_replace(0..1, "Hey, y", 3); // Another author
    assert_eq!(cord.string, "Hey, yo! World!");
    assert_eq!(cord.runs(), 3);
    assert_eq!(cord.run_authors(0), vec![3, 2, 1]);
    assert_eq!(cord.run_length(0), 6);
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 4);
    assert_eq!(cord.runs[2], (1, 1, 5));
}

#[test]
fn replace_end_of_a_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 13)],
    };

    cord.apply_replace(7..13, "their", 1); // Same author
    assert_eq!(cord.string, "Hello, their");
    assert_eq!(cord.runs, vec![(1, 1, 12)]);

    cord.apply_replace(10..12, "re.", 2); // Different author
    assert_eq!(cord.string, "Hello, there.");
    assert_eq!(cord.runs(), 2);
    assert_eq!(cord.runs[0], (1, 1, 10));
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 3);

    cord.apply_replace(12..13, "!", 3); // Another author
    assert_eq!(cord.string, "Hello, there!");
    assert_eq!(cord.runs(), 3);
    assert_eq!(cord.runs[0], (1, 1, 10));
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 2);
    assert_eq!(cord.run_authors(2), vec![3, 2, 1]);
    assert_eq!(cord.run_length(2), 1);
}

#[test]
fn replace_within_run() {
    let mut cord = Cord {
        string: "Hello, world!".to_string(),
        runs: vec![(1, 1, 13)],
    };

    cord.apply_replace(1..5, "ey", 1); // Same author
    assert_eq!(cord.string, "Hey, world!");
    assert_eq!(cord.runs, vec![(1, 1, 11)]);

    cord.apply_replace(1..3, "owdy", 2); // Different author
    assert_eq!(cord.string, "Howdy, world!");
    assert_eq!(cord.runs(), 3);
    assert_eq!(cord.runs[0], (1, 1, 1));
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 4);
    assert_eq!(cord.runs[2], (1, 1, 8));

    cord.apply_replace(4..5, "'y", 3); // Another author
    assert_eq!(cord.string, "Howd'y, world!");
    assert_eq!(cord.runs(), 4);
    assert_eq!(cord.runs[0], (1, 1, 1));
    assert_eq!(cord.run_authors(1), vec![2, 1]);
    assert_eq!(cord.run_length(1), 3);
    assert_eq!(cord.run_authors(2), vec![3, 2, 1]);
    assert_eq!(cord.run_length(2), 2);
    assert_eq!(cord.runs[3], (1, 1, 8));
}

#[test]
fn replace_across_runs() {
    let cord = Cord {
        string: "aaaabbbbccccdddd".to_string(),
        runs: vec![(1, 1, 4), (1, 2, 4), (1, 3, 4), (1, 4, 4)],
    };

    // First author at start, equal replacement
    let mut c = cord.clone();
    c.apply_replace(0..6, "------", 1);
    assert_eq!(c.string, "------bbccccdddd");
    assert_eq!(c.runs[0], (1, 1, 6));
    assert_eq!(c.runs[1], (1, 2, 2));
    assert_eq!(c.runs[2], (1, 3, 4));
    assert_eq!(c.runs[3], (1, 4, 4));

    // First author at start, shorter, replacement
    let mut c = cord.clone();
    c.apply_replace(0..6, "----", 1);
    assert_eq!(c.string, "----bbccccdddd");
    assert_eq!(c.runs[0], (1, 1, 4));
    assert_eq!(c.runs[1], (1, 2, 2));
    assert_eq!(c.runs[2], (1, 3, 4));
    assert_eq!(c.runs[3], (1, 4, 4));

    // First author at start, longer replacement
    let mut c = cord.clone();
    c.apply_replace(0..6, "--------", 1);
    assert_eq!(c.string, "--------bbccccdddd");
    assert_eq!(c.runs[0], (1, 1, 8));
    assert_eq!(c.runs[1], (1, 2, 2));
    assert_eq!(c.runs[2], (1, 3, 4));
    assert_eq!(c.runs[3], (1, 4, 4));

    // New author, shorter replacement in middle
    let mut c = cord.clone();
    c.apply_replace(6..10, "--", 5);
    assert_eq!(c.string, "aaaabb--ccdddd");
    assert_eq!(c.runs[0], (1, 1, 4));
    assert_eq!(c.runs[1], (1, 2, 2));
    assert_eq!(c.run_authors(2), vec![5, 2]);
    assert_eq!(c.run_length(2), 2);
    assert_eq!(c.runs[3], (1, 3, 2));
    assert_eq!(c.runs[4], (1, 4, 4));

    // New author, wide, longer replacement in middle
    let mut c = cord.clone();
    c.apply_replace(1..15, "---------------", 5);
    assert_eq!(c.string, "a---------------d");
    assert_eq!(c.runs[0], (1, 1, 1));
    assert_eq!(c.run_authors(1), vec![5, 1]);
    assert_eq!(c.run_length(1), 3);
    assert_eq!(c.run_authors(2), vec![5, 2]);
    assert_eq!(c.run_length(2), 4);
    assert_eq!(c.run_authors(3), vec![5, 3]);
    assert_eq!(c.run_length(3), 4);
    assert_eq!(c.runs[4], (1, 5, 4)); // Note additional 4 chars here with only new author
    assert_eq!(c.runs[5], (1, 4, 1));

    // New author, as above but ending on boundary of existing run
    let mut c = cord.clone();
    c.apply_replace(1..12, "---------------", 5);
    assert_eq!(c.string, "a---------------dddd");
    assert_eq!(c.runs[0], (1, 1, 1));
    assert_eq!(c.run_authors(1), vec![5, 1]);
    assert_eq!(c.run_length(1), 3);
    assert_eq!(c.run_authors(2), vec![5, 2]);
    assert_eq!(c.run_length(2), 4);
    assert_eq!(c.run_authors(3), vec![5, 3]);
    assert_eq!(c.run_length(3), 4);
    assert_eq!(c.runs[4], (1, 5, 4));
    assert_eq!(c.runs[5], (1, 4, 4));
}

// Merge two cords. Used for testing that merged value is correct
// and that does not panic due to invalid slots
fn merge_cords(s1: &str, s2: &str) {
    let mut c1 = Cord::with_author(s1, 0);
    let c2 = Cord::from(s2);

    let ops = c1.create_ops(&c2);
    //println!("{ops:?}");
    c1.apply_ops(ops, 1);

    assert_eq!(c1.string, s2)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn proptest_alpha_num(s1 in "[a-zA-Z0-9]*", s2 in "[a-zA-Z0-9]*") {
        merge_cords(&s1, &s2);
    }

    #[test]
    fn proptest_unicode(s1 in "\\PC*", s2 in "\\PC*") {
       merge_cords(&s1, &s2);
    }
}

// The following are regression tests for problems found from proptests

#[test]
fn no_zero_run_lengths() {
    merge_cords("", "A");
}

#[test]
fn unicode_merges() {
    merge_cords("", "🌀");
    merge_cords("🌀", "");
    merge_cords("🌀", "a");
}
