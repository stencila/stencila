// TODO: it should be easier stub out the engine
// ATM  the adapters heaviy use the engine's internal API to update
// the engine's internal model of these documents.
export default class StubEngine {
  run() {}
  addDocument() {
    return new StubEngineArticleModel()
  }
  addSheet() {
    return new StubEngineSheetModel()
  }
  on() {}
}

class StubEngineArticleModel {
  setAutorun() {}
  updateCell() {}
}

class StubEngineSheetModel {
  insertRows() {}
  deleteRows() {}
  insertCols() {}
  deleteCols() {}
  updateCell() {}
}
