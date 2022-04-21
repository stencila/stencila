use serde::{Deserialize, Serialize};

// Generate Rust types for the Google Docs API
//
// The `./gdoc.json` file was downloaded manually from https://docs.googleapis.com/$discovery/rest?version=v1
// and the top level property "schemas" changed to "definitions".
//
// That URL requires authentication (?!) and is not expected to change much. That's why we haven't bothered
// creating a curl+sed script for doing the above.
schemafy::schemafy!("src/gdoc.json");
