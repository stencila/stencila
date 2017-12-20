import { uuid } from 'substance'

export default class CellIssue {

  constructor({ type, msg, severity, cellId, data }) {
    if(!type) {
      return new Error("'type' is mandatory")
    }
    if(!cellId) {
      return new Error("'cellId' is mandatory")
    }
    // a generated id to identify the issue
    this.id = uuid()
    this.type = type
    this.msg = msg
    this.severity = severity || 0
    this.cellId = cellId
    this.data = data
  }

  get key() {
    return `${this.type}#${this.cellId}`
  }

  get message() {
    return this.msg
  }

  isError() {
    return this.severity === 2
  }

  isWarning() {
    return this.severity === 1
  }

  isCellIssue() {
    return true
  }
}