# Stencila Sheets

Next:

## Tasks:

- XML-data model close to HTML table
- TableSelection
  - Drag Multiselection
    - Anchor cell should be highlighted
    - Typing right into Anchor Cell
    - Backspace should clear selected cells
    - Pasted cells should be inserted at anchor cell
      - After paste the target selection is equal to the copied selection
      - Google Docs does magic tiling/repeating expansion when target selection bigger then copied selection
- Interaction with Cells (select multiple cells, ENTER allows editing, ~same behaviour as Google Sheets)
- Resize Columns/Rows
  - Questionable?
- Drag down sequences
  - Detect increment (1,3,5..., 1,2,3,...)
  - Otherwise we just repeat the selected pattern (better than GDocs)
- Create/Remove Column/Row
- Explicit setting of cell type (currency, float rounding etc.)
- Autocompletion of function names (+parameter hints etc) and variables
- Visual clues when referencing cells
- Click to Chart (like in Google Docs)
  - Range selection and click create chart
  - Standard statistics are shown (min, max, sum)
  - Edit Range manually (define how many header rows there are)
  - Suggest a set of standard plots according data selection
  - Drag suggest chart into Sheet
- Chart anchoring
  - Floating Cell: Not part of the sheet, but an extra overlay cell containing a formula and rendering at x,y relative to anchor cell
  - Use anchors as a place for formulas
- Visual chart editing/configuration (maybe plotly has something)
  Looking at GSheets (minus styling related):
  - Data:
    - chart type
    - stacking
    - data range
    - axis labels
    - some transformations (transpose)
    - 'Use Row 47 as headers'
    - 'Use column D as labels'
  - Customizations:
    - Chart axis titles
    - Error bars, Data labels for each series
    - Axis: min, max, units (auto, or custom)
    - ...
- Cell formatting
  - Styles on whole cell (semantically annotated: e.g. heading, body)
  - Ability to set alignment (e.g. center, also vertically)
  - Borders (Google Sheets allows speciying border-left,right,top for each cell separately)
- Pivot Table
  - This is some kind of data table interface that Excel/Google offer
  - Ability to sort and filter
  - Column names + types should be known

## Differences DataTable vs Sheet:

- DataTable has explicit column_names (like CSV header row)
- DataTable has one datatype per column as opposed to type per cell in Sheet
