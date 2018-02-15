class CellIssue {
  constructor(cellId, type, msg, severity, data) {
    this.cellId = cellId
    this.type = type
    this.msg = msg

    if(!cellId || !type || !msg) {
      return new Error('cellId, type and message are mandatory')
    }

    if(severity !== null && typeof severity === 'object') {
      data = severity
      severity = 0
    }

    this.severity = severity || 0
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

export default CellIssue
