/**
 * An NPM install script (run on `npm install` and `npm ci`) to download
 * binary addons for the platform from GitHub release.
 *
 * The file name function includes more architectures than are currently
 * supported but are kept here for reference.
 */

const { readFileSync, createWriteStream } = require("fs");
const path = require("path");
const https = require("https");
const { createGunzip } = require("zlib");

const version = JSON.parse(
  readFileSync(path.join(__dirname, "package.json"))
).version;

const { platform, arch } = process;

const target = (() => {
  switch (platform) {
    case "win32":
      switch (arch) {
        case "x64":
          return "win32-x64-msvc";
        case "ia32":
          return "win32-ia32-msvc";
        case "arm64":
          return "win32-arm64-msvc";
        default:
          throw new Error(`Unsupported architecture on Windows: ${arch}`);
      }
    case "darwin":
      switch (arch) {
        case "x64":
          return "darwin-x64";
        case "arm64":
          return "darwin-arm64";
        default:
          throw new Error(`Unsupported architecture on macOS: ${arch}`);
      }
    case "linux":
      switch (arch) {
        case "x64":
          return "linux-x64-gnu";
        case "arm64":
          return "linux-arm64-gnu";
        case "arm":
          return "linux-arm-gnueabihf";
        default:
          throw new Error(`Unsupported architecture on Linux: ${arch}`);
      }
    default:
      throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
  }
})();

const url = `https://github.com/stencila/stencila/releases/download/v${version}/stencila.${target}.node.gz`;

function followRedirects(url, callback) {
  const req = https.get(url, (res) => {
    if (res.statusCode === 302 || res.statusCode === 301) {
      followRedirects(res.headers.location, callback);
    } else {
      callback(res);
    }
  });

  req.on("error", (err) => {
    throw new Error("Error downloading the file:", err);
  });
}

followRedirects(url, (res) => {
  if (res.statusCode !== 200) {
    throw new Error(`Failed to download ${url}: ${res.statusCode}`);
  }

  res
    .pipe(createGunzip())
    .pipe(createWriteStream(`stencila.node`))
    .on("finish", () => {
      console.log(`File "${target}" downloaded and extracted.`);
      process.exit(0);
    });
});
