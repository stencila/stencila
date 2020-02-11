/**
 * Run visual regression tests.
 *
 * Note that `npm run test:build` must be run before running these tests.
 * That script runs Parcel to build the `test/build` directory with all the
 * HTML, compiled CSS and JS used for visual regression testing in it.
 */

const fs = require('fs')
const http = require('http')
const { env: {staticDir, baseUrl, examples, diff} } = require('./config.js')
const assert = require('assert')

// Get hostname and port for doing connectivity tests
const {hostname, port} = new URL(baseUrl)

/**
 * Takes an array of visual regression comparison results, and checks if all
 * are within difference tolerances.
 *
 * @param {array {isWithinMisMatchTolerance: boolean}} results
 * @returns boolean
 */
const allScreenshotsPass = results =>
  results.reduce(
    (pass, result) => pass && result.isWithinMisMatchTolerance === true,
    true
  )

describe('visual regressions: ', () => {
  it(`needs the build folder`, () => {
    assert.ok(
      fs.existsSync(staticDir),
      `${staticDir} does not exist. Did you run 'npm run test:build' first?`
    )
  })

  describe(`runs on each HTML file`, () => {
    const files = fs.readdirSync(staticDir).filter(file => file.endsWith('.html'))
    files.forEach(file => {
      const path = `/${file}`

      it(`${file}: can be got`, () => {
        const req = http.get({ hostname, port, path }, res => {
          assert.equal(res.statusCode, '200')
        })
        req.on('error', error => assert.fail(error))
        req.end()
      })

      // A pseudo-test that is helpful for debugging the page
      // that the screen-shotting actually sees. To use it un-skip it.
      it.skip(`${file}: can be browsed`, async () => {
        console.log(`Browse for 60s before the robots 🤖 take control: ${baseUrl}${path}`)
        await new Promise(resolve => setTimeout(resolve, 60000))
      })

      it(`${file}: screenshots have not changed`, async () => {
        await browser.url(path)
        const results = await browser.checkDocument()

        assert.ok(
          allScreenshotsPass(results),
          `Styles differ from current references. Please see ${diff} for differences`
        )
      })
    })
  })
})
