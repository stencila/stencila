const fs = require('fs')
const path = require('path')
const { env, normalizeName } = require('./config.js')
const assert = require('assert')

const isHTML = file => file.endsWith('.html')

// Find HTML files stored the examples folder for testing
const files = fs.readdirSync(env.examples).filter(isHTML)

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

describe('visual regression: ', () => {
  files.forEach(function(file) {
    it(`compare screenshot of ${file}`, async () => {
      await browser.url('/' + file)
      const results = await browser.checkDocument()

      assert.ok(
        allScreenshotsPass(results),
        `Styles differ from current references. Please see ${env.diff} for differences`
      )
    })
  })
})
