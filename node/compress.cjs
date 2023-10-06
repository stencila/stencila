/**
 * An NPM script to compress Node.js binary addon built by NAPI-RS
 * before it uploaded to GitHub releases
 */

const { createReadStream, createWriteStream, readdirSync } = require("fs");
const path = require("path");
const { createGzip } = require("zlib");

let addon;
for (const file of readdirSync(__dirname)) {
  if (file.startsWith("stencila.") && file.endsWith(".node")) {
    addon = file;
    break;
  }
}

createReadStream(path.join(__dirname, addon))
  .pipe(createGzip())
  .pipe(createWriteStream(path.join(__dirname, `${addon}.gz`)))
  .on("finish", () => console.log(`Addon ${addon} successfully compressed`));
