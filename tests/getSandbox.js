import { inBrowser, DefaultDOMElement } from 'substance'

export default function getSandbox(t) {
  // when running with substance-test we get
  // a sandbox for each test
  if (t.sandbox) return t.sandbox
  // otherwise we create our own DOM
  let htmlDoc = DefaultDOMElement.parseHTML('<html><body></body></html>')
  return htmlDoc.find('body')
}