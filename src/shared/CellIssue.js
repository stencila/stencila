class CellIssue {
  constructor(cellId, type, msg, severety, data) {
    this.cellId = cellId
    this.type = type
    this.msg = msg

    if(!cellId || !type || !msg) {
      return new Error('cellId, type and message are mandatory')
    }

    if(severety !== null && typeof severety === 'object') {
      data = severety
      severety = 0
    }

    this.severety = severety || 0
    this.data = data
  }

  get key() {
    return `${this.type}#${this.cellId}`
  }

  get message() {
    return this.msg
  }

  isError() {
    return this.severety === 2
  }

  isWarning() {
    return this.severety === 1
  }

  isCellIssue() {
    return true
  }
}

export default CellIssue
