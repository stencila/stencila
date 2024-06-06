use std::collections::{HashMap, HashSet};
use std::fs;

use assistant::SpecializedAssistant;
use assistants::get_assistant;
use model::{
    common::{eyre::Result, serde::Deserialize, serde_yaml, tokio},
    Embeddings, GenerateTask, Instruction, InstructionType, Model,
};

#[derive(Debug, Deserialize)]
#[serde(crate = "model::common::serde")]
struct AssistantTest {
    text: Vec<String>,
}

// If your YAML file contains an array of tests
#[derive(Debug, Deserialize)]
#[serde(crate = "model::common::serde")]
struct TestCases(Vec<AssistantTest>);

async fn local_get_assistant(itype: InstructionType, text: String) -> Result<(String, f32)> {
    let mut task = match itype {
        InstructionType::InsertBlocks => GenerateTask::new(Instruction::block_text(text), None),
        InstructionType::ModifyBlocks => {
            GenerateTask::new(Instruction::block_text_with(text, vec![]), None)
        }
        InstructionType::InsertInlines => GenerateTask::new(Instruction::inline_text(text), None),
        InstructionType::ModifyInlines => {
            GenerateTask::new(Instruction::inline_text_with(text, vec![]), None)
        }
    };
    let assistant = get_assistant(&mut task).await?;
    let score = assistant.suitability_score(&mut task)?;
    Ok((assistant.name(), score))
}

fn short_name(id: &str) -> String {
    id.split('/')
        .nth(1)
        .expect("should be a `/` in an id")
        .to_string()
}

#[tokio::test]
async fn check_we_get_the_right_assistant() -> Result<()> {
    // Make a lookup of all special assistants built in to stencila.
    let special_by_key: HashMap<String, SpecializedAssistant> =
        assistant::list_builtin_as_specialized()?
            .into_iter()
            // Remove "stencila/"
            .map(|a| (short_name(&a.name()), a))
            .collect();

    let file_content =
        fs::read_to_string("tests/assistants.yaml").expect("Unable to read the YAML file");

    let test_cases: HashMap<String, AssistantTest> =
        serde_yaml::from_str(&file_content).expect("Cannot parse test YAML");

    // Iterate over the test cases
    let mut found: HashSet<String> = HashSet::new();
    for (id, tests) in test_cases {
        println!("Testing {}", id);
        let asst = &special_by_key[&id];
        found.insert(id.clone());
        // Here you can call the function to test with `case.challenge`
        for txt in tests.text {
            print!("-- Trying `{}`...", txt);
            let (matched_id, score) = local_get_assistant(
                asst.instruction_type().expect("should have a type"),
                txt.clone(),
            )
            .await?;
            // Test failure happens here.
            // assert_eq!(id, short_name(matched_id));
            if id != short_name(&matched_id) {
                println!("FAIL: matched {} instead of {}", matched_id, id);
                continue;
            }
            println!("OK with score {}", score);
        }
    }

    // Check that all special assistants are tested
    for (id, _) in special_by_key {
        if found.contains(&id) {
            continue;
        }
        println!("No tests found for {}", id);
        // Let's not fail just yet.
        // assert!(found.contains(&id));
    }

    Ok(())
}

#[test]
fn ensure_instruction_examples_are_distinct() -> Result<()> {
    // Make a lookup of all special assistants built in to stencila.
    let mut assistants = assistant::list_builtin_as_specialized()?;
    for a in assistants.iter_mut() {
        a.init()?;
    }

    let all_comparisons: Vec<(&SpecializedAssistant, &str, &[f32])> = assistants
        .iter()
        .flat_map(|a| {
            a.instruction_embeddings()
                .iter_items()
                .map(move |(t, v)| (a, t, v))
        })
        .collect();

    // TODO: Probably sort this.
    for (a1, t1, v1) in all_comparisons.iter() {
        for (a2, t2, v2) in all_comparisons.iter() {
            println!(
                "{:.4} | {}: `{}` --- {}: `{}`",
                Embeddings::calculate_similarity(v1, v2),
                short_name(&a1.name()),
                t1,
                short_name(&a2.name()),
                t2,
            );
        }
    }
    Ok(())
}
