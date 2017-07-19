# Stencila Sheets

## Data Model

- XML-data model close to HTML table
- [?] Converters to other formats 

## Sheet UI

- [?] Start with smaller empty sheet (~Pages)
- [?] vs: Start with a large empty sheet (~Excel, GSheet)
- Show a row with column ids
- Show a column with

## Table Selection

- Create a selection range via dragging
- Anchor cell is highlighted
- Typing with a range selection overwrites anchor cell
- Backspace/Delete clears all selected cells
- Pasting cells inserts content starting at the anchor cell
  - After paste the target selection is equal to the copied 
    selection
  - [?] GSheets does magic tiling / repeating 
    when target selection is bigger then the copied selection

## Cell Interaction

- Set type of a single cell (currency, float rounding etc.)

## Row / Column Interaction

- Set column type
- Insert row / column before / after
- Delete row / column

## Cell Editing

- ENTER allows editing (~same behavior as Google Sheets)
- Autocompletion of function names (+parameter hints etc) and variables
- Visual clues when referencing cells
- Cell style for whole cell (semantically, e.g. heading, body)
- Ability to set horizontal and vertical alignment (e.g. center, also vertically)
- Borders
  - left, top, right, bottom, 
  - inner, outer
  - double

## Table Interaction

- [?] Resize columns / rows. 
  
  This is kind of formatting. You would not do that in latex

- Extending sequences
  - Detect increment (1,3,5..., 1,2,3,...)
  - Otherwise just repeat the selected pattern (better than GDocs)

- Create / remove column / row

## Charts

- Charts for clickers (inspired by GSheets)
  - Range selection and click create chart
  - [?] In GSheets standard statistics are shown (min, max, sum)
  - Edit range manually 
  - [?] Define how many header rows which are used as labels
  - Suggest a set of standard plots according data selection
  - Drag chart into Sheet
- Chart anchoring
  - Insert a formula into the anchor cell
  - Position floating chart relative to anchor cell
- Visual chart configuration (Inspired by GSheets)
  - Change chart type
  - Stacking of multiple graphs
  - Edit the data range
  - Change the axis labels
  - Configure axis: min, max, units (auto, or custom)
  - Switch x/y
  - Option to toggle use headers from data
    *"Use Row 47 as headers"*
  - Option to use labels from data
    *"Use column D as labels"*
  - [?] Show error bars
  - [?] Show data labels

## Data Tables / Pivot Tables

This is some kind of data table interface that Excel/Google offer
We don't like how Pivot Tables are embedded in Excel/Google Sheets,
which is confusing.
There must be a row with column names.
Every column has a type, cells can not override the column type.

- Ability to sort and filter
- Change a column's name
- Change a column's type