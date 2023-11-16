/**
 * Run accessibility tests on HTML generated for examples using pa11y
 * 
 * This test could live elsewhere but it seems to make most sense for it
 * to reside close to other tests related to codec output.
 */

const glob = require("glob");
const pa11y = require("pa11y");
const path = require("path");

const examples = path.join(__dirname, "../../../../examples/nodes");
const files = glob.globSync("*/*.standalone.html", {
  cwd: examples,
});

for (const file of files) {
  pa11y(path.join(examples, file), {
    // Necessary for running on Ubuntu 22.04 and above.
    // See https://github.com/pa11y/pa11y/issues/662
    chromeLaunchConfig: {
      ignoreHTTPSErrors: false,
      executablePath: process.env.CI
        ? "/opt/hostedtoolcache/chromium/latest/x64/chrome" // Github Actions Ubuntu 22.04
        : "/usr/bin/google-chrome", // Local Ubuntu 22.04
    },
    ignore: [
      // TODO fix this issue and remove the ignore
      'WCAG2AA.Principle1.Guideline1_1.1_1_1.H37'
    ]
  }).then(async (results) => {
    if (results.issues.length > 0) {
      console.error(results);
      // Exit on the first file with issues
      process.exit(results.issues.length);
    }
  });
}
