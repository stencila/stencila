/**
 * Run visual regression tests.
 *
 * Note that `npm run test:build` must be run before running these tests.
 * That script runs Parcel to build the `test/build` directory with all the
 * HTML, compiled CSS and JS used for visual regression testing in it.
 */

const assert = require('assert')
const fs = require('fs')
const http = require('http')
const {
  env: { staticDir, baseUrl, diff }
} = require('./config.js')
const { examples } = require('./build/examples')
const { themes } = require('./build/themes')

// The examples to use for visual regression tests
// It's generally better to only use small examples, as
// larger ones take up time and space
const EXAMPLES = [examples.articleKitchenSink]

// The themes to be tested. Defaults to all
const THEMES = Object.keys(themes)

// Get hostname and port for doing connectivity tests
const { hostname, port } = new URL(baseUrl)

describe('visual regressions: ', () => {
  it(`needs the build folder`, () => {
    assert.ok(
      fs.existsSync(staticDir),
      `${staticDir} does not exist. Did you run 'npm run test:build' first?`
    )
  })

  it(`demo page can be got`, () => {
    const req = http.get({ hostname, port }, res => {
      assert.equal(res.statusCode, '200')
    })
    req.on('error', error => assert.fail(error))
    req.end()
  })

  describe(`runs over examples and themes: `, () => {
    EXAMPLES.forEach(example => {
      THEMES.forEach(theme => {
        const path = `?example=${example}&theme=${theme}&header=false`

        // A pseudo-test that is helpful for debugging the page
        // that the screen-shotting actually sees. To use it un-skip it.
        it.skip(`${path}: can be browsed`, async () => {
          console.log(
            `Browse for 60s before the robots 🤖 take control: ${baseUrl}${path}`
          )
          await new Promise(resolve => setTimeout(resolve, 60000))
        })

        it(`${path}: screenshots have not changed`, async () => {
          await browser.url(path)
          const results = await browser.checkDocument()

          /**
           * Takes an array of visual regression comparison results, and checks if all
           * are within difference tolerances.
           */
          const allScreenshotsPass = results =>
            results.reduce(
              (pass, result) =>
                pass && result.isWithinMisMatchTolerance === true,
              true
            )
          assert.ok(
            allScreenshotsPass(results),
            `Styles differ from current references. Please see ${diff} for differences`
          )
        })
      })
    })
  })
})
