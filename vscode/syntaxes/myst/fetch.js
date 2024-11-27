/**
 * Fetches MyST language grammar, snippets etc from main branch of official repo
 *
 * These files are 'vendored' (i.e. committed) into this repo.
 */

const fs = require("fs").promises;
const path = require("path");
const https = require("https");

const files = [
  [
    `https://raw.githubusercontent.com/jupyter-book/vscode-mystmd/refs/heads/main/syntaxes/myst.tmLanguage.json`,
    "myst.tmLanguage.json",
  ],
  [
    `https://raw.githubusercontent.com/jupyter-book/vscode-mystmd/refs/heads/main/language-configuration.json`,
    "configuration.json",
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
