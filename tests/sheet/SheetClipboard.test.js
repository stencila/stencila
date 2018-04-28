import test from 'tape'
import { DefaultDOMElement as DOM, flatten } from 'substance'
import SheetClipboard from '../../src/sheet/SheetClipboard'
import StubEngine from '../util/StubEngine'
import setupSheetSession from '../util/setupSheetSession'
import { setSheetSelection } from '../util/sheetTestHelpers'

test('SheetClipboard: copy', (t) => {
  let fixture = simple()
  let { sheetSession, sheetClipboard } = _setup(simple())
  setSheetSelection(sheetSession, 'A1:C3')
  let cells = fixture.cells.slice(0, 3)
  let evt = new ClipboardEvent()
  sheetClipboard.onCopy(evt)
  let data = evt.clipboardData.data
  let plainText = data['text/plain']
  let html = data['text/html']
  t.ok(plainText, 'there should be a plain text copy')
  t.equal(plainText, cells.map(row => row.join('\t')).join('\n'), '.. encoded as tab separated values')
  t.ok(html, 'there should be an html copy')
  let dom = DOM.parseHTML(html)
  t.equal(dom.findAll('tr').length, 3, '.. with 3 <tr>')
  t.equal(dom.findAll('td').length, 9, '.. with 9 <td>')
  t.deepEqual(
    dom.findAll('td').map(td => td.textContent),
    flatten(cells),
    '.. and correct cell content'
  )
  t.end()
})

function simple() {
  return {
    id: 'sheet',
    path: 'sheet.xml',
    type: 'sheet',
    name: 'My Sheet',
    columns: [{ name: 'x' }, { name: 'y' }, { name: 'z' }],
    cells: [
      ['1', '2', '3'],
      ['4', '5', '6'],
      ['7', '8', '9'],
      ['10', '11', '12']
    ]
  }
}

function _setup(sheetData) {
  let { sheetSession, sheet } = setupSheetSession(sheetData, new StubEngine())
  let sheetClipboard = new SheetClipboard(sheetSession)
  return { sheetSession, sheet, sheetClipboard }
}

class ClipboardEventData {
  constructor() {
    this.data = {}
  }

  getData(format) {
    return this.data[format]
  }

  setData(format, data) {
    this.data[format] = data
  }

  get types() {
    return Object.keys(this.data)
  }
}

class ClipboardEvent {
  constructor() {
    this.clipboardData = new ClipboardEventData()
  }
  preventDefault() {}
  stopPropagation() {}
}