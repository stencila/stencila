/**
 * Fetches Cypher Query Language grammar from https://github.com/jakeboone02/cypher-query-language
 * 
 * These files are 'vendored' (i.e. committed) into this repo.
 */

const fs = require("fs").promises;
const path = require("path");
const https = require("https");

const tag = "v2.0.0";

const files = [
  [
    `https://raw.githubusercontent.com/jakeboone02/cypher-query-language/refs/tags/${tag}/LICENSE.txt`,
    "LICENSE.txt",
  ],
  [
    `https://raw.githubusercontent.com/jakeboone02/cypher-query-language/refs/tags/${tag}/syntaxes/cypher.tmLanguage`,
    "cypher.tmLanguage",
  ],
  [
    `https://raw.githubusercontent.com/jakeboone02/cypher-query-language/refs/tags/${tag}/cypher.configuration.json`,
    "cypher.configuration.json",
  ],
];

async function downloadFile(url, outputPath) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (response.statusCode !== 200) {
          reject(
            new Error(`Failed to download ${url}: ${response.statusCode}`)
          );
          return;
        }

        let data = "";
        response.on("data", (chunk) => (data += chunk));
        response.on("end", () => {
          fs.writeFile(outputPath, data)
            .then(() => resolve())
            .catch(reject);
        });
      })
      .on("error", reject);
  });
}

async function main() {
  const scriptDir = __dirname;

  try {
    for (const [url, filename] of files) {
      const outputPath = path.join(scriptDir, filename);
      console.log(`Downloading ${url} to ${outputPath}...`);
      await downloadFile(url, outputPath);
      console.log(`Successfully downloaded ${filename}`);
    }
  } catch (error) {
    console.error("Error downloading files:", error);
    process.exit(1);
  }
}

main();
