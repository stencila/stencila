const fs = require('fs')
const http = require('http')
const { env: {baseUrl, examples, diff} } = require('./config.js')
const assert = require('assert')

// Get hostname and port for doing connectivity tests
const {hostname, port} = new URL(baseUrl)

// Find HTML files stored the examples folder for testing
const files = fs.readdirSync(examples).filter(file => file.endsWith('.html'))

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
  files.forEach(file => {
    const path = `/examples/${file}`

    it(`${file}: can be got`, () => {
      const req = http.get({ hostname, port, path }, res => {
        assert.equal(res.statusCode, '200')
      })
      req.on('error', error => assert.fail(error))
      req.end()
    })

    // A pseudo-test that is helpful for debugging the page
    // that the screen-shotting actually sees. To use it unskip it.
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
