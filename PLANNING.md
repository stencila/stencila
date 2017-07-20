# Stencila Sheets

## Data Model

- XML-data model close to HTML table
- [?] Converters to other formats 

## Sheet UI

- [?] Start with smaller empty sheet (~Pages)
- [?] vs: Start with a large empty sheet (~Excel, GSheet)
- Show first row with column ids and first column with row ids
- Display a row / column menu on right click (what about keyboard?)
- Select whole row / column when clicked on row / column label
- Select multiple rows / columns using Shift+MouseLeft or Shift+Down/Up/Left/Right
- [?] Append row / column (~Pages)
- Show content of current anchor cell in an extra bar
- Edit sheet title
- Edit sheet name (used for referencing)

## Table Selection

The *Anchor Cell* is the cell where a range selection 
was started. 

- Select a single cell on mouse down
- Create a range selection via dragging 
- Create a range selection using Shift+LEFT/RIGHT/UP/DOWN
- Navigate using LEFT/RIGHT/UP/DOWN, and TAB/Shift+TAB
- Highlight the anchor cell
- Typing with a range selection overwrites anchor cell
- Backspace/Delete clears all selected cells
- Pasting cells inserts content starting at the anchor cell
  - After paste the target selection is equal to the copied 
    selection
  - [?] GSheets does magic tiling / repeating 
    when target selection is bigger then the copied selection

## Cell Interaction

- Display a cell context menu on right click (what about keyboard?)
- Set type + formatting of a single cell (units, precision, etc.)
- Go inside a cell on ENTER or DoubleClick 
- Discard cell changes on ESC and step out selecting the cell
- Conditional formatting
- [?] Data validation

## Row / Column Interaction

- Insert row / column before / after
- Delete row / column
- Clear row / column
- [?] Hide row / column
- Set type + formatting (units, precision, etc.) for the whole row / column
- [?] Drag a row / column to change the order
- [?] Conditional formatting
- [?] Data validation

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

- Change a column's name
- Change a column's type
- Add/remove new column
- Compatibility with Data Packages (http://frictionlessdata.io/data-packages/)
- Ability to sort and filter
- Realtime collab should be possible
- Pressing ENTER takes you to cell below, when on last row, a new row is created
- Paging mechanism if there are many rows
- Dialog for importer (e.g. to ask wether a CSV
- [?] SQL backed filtering, sorting

Similar projects: (https://airtable.com/, https://fieldbook.com/)
