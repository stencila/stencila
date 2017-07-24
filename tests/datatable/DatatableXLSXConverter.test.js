import test from 'tape'

import DatatableXLSXConverter from '../../src/datatable/DatatableXLSXConverter'
import MemoryStorer from '../../src/host/MemoryStorer'

test('DatatableXLSXConverter:match', t => {
  let c = new DatatableXLSXConverter()

  t.plan(2)
  
  c.match('data.xlsx').then(result => {
    t.ok(result)
  }, 'a XLSX file')

  c.match('data.csv').then(result => {
    t.notOk(result)
  }, 'not a XLSX file')
})

test('DatatableXLSXConverter:import', t => {
  let c = new DatatableXLSXConverter()
  let storer = new MemoryStorer() // eslint-disable-line
  t.throws(c.import, 'DatatableXLSXConverter.import() not yet implemented')
  t.end()
})

test('DatatableXLSXConverter:_importWorksheetToDatatable', t => {
  // This XML was obtained from a .xlxs file produced by Open Office
  let worksheet = `<worksheet>
  <sheetPr filterMode="false">
    <pageSetUpPr fitToPage="false" />
  </sheetPr>
  <dimension ref="A1:B4" />
  <sheetViews>
    <sheetView windowProtection="false" showFormulas="false" showGridLines="true" showRowColHeaders="true" showZeros="true" rightToLeft="false" tabSelected="true" showOutlineSymbols="true" defaultGridColor="true" view="normal" topLeftCell="A1" colorId="64" zoomScale="100" zoomScaleNormal="100" zoomScalePageLayoutView="100" workbookViewId="0">
      <selection pane="topLeft" activeCell="B1" activeCellId="0" sqref="B1" />
    </sheetView>
  </sheetViews>
  <sheetFormatPr defaultRowHeight="12.8" />
  <cols>
    <col collapsed="false" hidden="false" max="1025" min="1" style="0" width="11.5204081632653" />
  </cols>
  <sheetData>
    <row r="1" customFormat="false" ht="12.8" hidden="false" customHeight="false" outlineLevel="0" collapsed="false">
      <c r="A1" s="0" t="s">
        <v>0</v>
      </c>
      <c r="B1" s="0" t="s">
        <v>1</v>
      </c>
    </row>
    <row r="2" customFormat="false" ht="12.8" hidden="false" customHeight="false" outlineLevel="0" collapsed="false">
      <c r="A2" s="0" t="s">
        <v>2</v>
      </c>
      <c r="B2" s="0" t="n">
        <v>1</v>
      </c>
    </row>
    <row r="3" customFormat="false" ht="12.8" hidden="false" customHeight="false" outlineLevel="0" collapsed="false">
      <c r="A3" s="1" t="s">
        <v>3</v>
      </c>
      <c r="B3" s="0" t="n">
        <v>2</v>
      </c>
    </row>
    <row r="4" customFormat="false" ht="12.8" hidden="false" customHeight="false" outlineLevel="0" collapsed="false">
      <c r="A4" s="0" t="s">
        <v>4</v>
      </c>
      <c r="B4" s="0" t="n">
        <v>3</v>
      </c>
    </row>
  </sheetData>
  <printOptions headings="false" gridLines="false" gridLinesSet="true" horizontalCentered="false" verticalCentered="false" />
  <pageMargins left="0.7875" right="0.7875" top="1.05277777777778" bottom="1.05277777777778" header="0.7875" footer="0.7875" />
  <pageSetup paperSize="9" scale="100" firstPageNumber="1" fitToWidth="1" fitToHeight="1" pageOrder="downThenOver" orientation="portrait" usePrinterDefaults="false" blackAndWhite="false" draft="false" cellComments="none" useFirstPageNumber="true" horizontalDpi="300" verticalDpi="300" copies="1" />
  <headerFooter differentFirst="false" differentOddEven="false">
    <oddHeader>&C&"Times New Roman,Regular"&12&A</oddHeader>
    <oddFooter>&C&"Times New Roman,Regular"&12Page &P</oddFooter>
  </headerFooter>
</worksheet>`.replace(/ {2}|\n/g,'')

  let sharedStrings = `<sst count="5" uniqueCount="5">
  <si>
    <t xml:space="preserve">field1</t>
  </si>
  <si>
    <t xml:space="preserve">field2</t>
  </si>
  <si>
    <t xml:space="preserve">a</t>
  </si>
  <si>
    <t xml:space="preserve">b</t>
  </si>
  <si>
    <t xml:space="preserve">c</t>
  </si>
</sst>`.replace(/ {2}|\n/g,'')

  let datatable = `<datatable>
  <fields>
    <field name="field1"/>
    <field name="field2"/>
  </fields>
  <values>
    <row>
      <value>a</value>
      <value>1</value>
    </row>
    <row>
      <value>b</value>
      <value>2</value>
    </row>
    <row>
      <value>c</value>
      <value>3</value>
    </row>
  </values>
</datatable>`.replace(/ {2}|\n/g,'')

  let c = new DatatableXLSXConverter()
  t.equal(
    c._importDatatableFromWorksheet(worksheet, sharedStrings).getOuterHTML(),
    datatable
  )
  t.end()
})

test('DatatableXLSXConverter:export', t => {
  let c = new DatatableXLSXConverter()
  t.throws(c.export, 'DatatableXLSXConverter.export() not yet implemented')
  t.end()
})
