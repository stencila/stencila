/**
  Immutable representation of a table selection

  @example

  Construct

  ```js
  var sel = new TableSelection({startRow: 0, startCol: 3, endRow: 2: endCol: 3})
  ```

  Modify

  ```js
  var newSel = sel.toJSON()
  newSel.endRow += 1
  newSel = new TableSelection(newSel)
  ```
*/
export default
class TableSelection {

  constructor(props) {
    this.startRow = props.startRow
    this.startCol = props.startCol
    this.endRow = props.endRow
    this.endCol = props.endCol
    Object.freeze(this)
  }

  isCollapsed() {
    return (this.startRow === this.endRow && this.startCol === this.endCol)
  }

  toString() {
    return "T[("+ this.startRow + "," + this.startCol + "), ("+ this.endRow + ", " + this.endCol +")]"
  }

  toJSON() {
    return {
      startRow: this.startRow,
      startCol: this.startCol,
      endRow: this.endRow,
      endCol: this.endCol
    }
  }

}
