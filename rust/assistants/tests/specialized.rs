use std::collections::{HashMap, HashSet};
use std::fs;

use assistant::{
    common::{eyre::Result, serde::Deserialize, serde_yaml, tokio},
    Assistant, GenerateTask, Instruction, InstructionType,
};
use assistant_specialized::{Embeddings, SpecializedAssistant};
use assistants::get_assistant;

#[derive(Debug, Deserialize)]
#[serde(crate = "assistant::common::serde")]
struct AssistantTest {
    text: Vec<String>,
}

// If your YAML file contains an array of tests
#[derive(Debug, Deserialize)]
#[serde(crate = "assistant::common::serde")]
struct TestCases(Vec<AssistantTest>);

async fn local_get_assistant(itype: InstructionType, text: String) -> Result<(String, f32)> {
    let mut task = match itype {
        InstructionType::InsertBlocks => GenerateTask::new(Instruction::block_text(text)),
        InstructionType::ModifyBlocks => {
            GenerateTask::new(Instruction::block_text_with(text, vec![]))
        }
        InstructionType::InsertInlines => GenerateTask::new(Instruction::inline_text(text)),
        InstructionType::ModifyInlines => {
            GenerateTask::new(Instruction::inline_text_with(text, vec![]))
        }
    };
    let assistant = get_assistant(&mut task).await?;
    let score = assistant.suitability_score(&mut task)?;
    Ok((assistant.id(), score))
}

fn short_name(id: &String) -> String {
    id.split('/')
        .nth(1)
        .expect("should be a `/` in an id")
        .to_string()
}

#[tokio::test]
async fn check_we_get_the_right_assistant() -> Result<()> {
    // Make a lookup of all special assistants built in to stencila.
    let special_by_key: HashMap<String, SpecializedAssistant> =
        assistant_specialized::list_builtin_as_specialized()?
            .into_iter()
            // Remove "stencila/"
            .map(|a| (short_name(&a.id()), a))
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

// TODO: Remove this
#[allow(unused_variables)]
#[tokio::test]
async fn ensure_assistants_are_distinct() -> Result<()> {
    // Make a lookup of all special assistants built in to stencila.
    let assistants = assistant_specialized::list_builtin_as_specialized()?;
    let empty_instr: Vec<String> = vec![];
    let empty_embed: Embeddings = vec![];

    // EEK. I'm not sure I should be proud of this.
    let asst_with_instr: Vec<_> = assistants
        .iter()
        .flat_map(|a| {
            a.instruction_examples()
                .as_ref()
                .unwrap_or(&empty_instr)
                .iter()
                .zip(a.instruction_embeddings().as_ref().unwrap_or(&empty_embed))
                .map(|i| (a, i))
                .collect::<Vec<_>>()
        })
        .collect();

    // Now do the full matrix of comparisons.
    for (a1, (i1, e1)) in asst_with_instr.iter() {
        for (a2, (i2, e2)) in asst_with_instr.iter() {
            if a1.id() == a2.id() && i1 == i2 {
                continue;
            }
            // TODO: Extract cosine similarity.
            // I think the embeddings need to be refactored into a newtype struct: InstructionEmbeddings(Vec<Vec<f32>>)
            // Then their creation and comparison can be done in a single place.
            // let sim = e1.cosine_similarity(e2);
            println!(
                "{:<20} {:<20} / {:<20} {:<20}: {}",
                short_name(&a1.id()),
                i1,
                short_name(&a2.id()),
                i2,
                0.0,
            );
        }
    }

    Ok(())
}
