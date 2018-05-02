
export function setCellLanguage(editorSession, cellId, newLanguage) {
  editorSession.transaction((tx) => {
    let cell = tx.get(cellId)
    let sourceCode = cell.find('source-code')
    sourceCode.attr({ language: newLanguage })
  }, { action: 'setCellLanguage'})
}

export function insertCell(editorSession) {
  editorSession.transaction(tx => {
    let sel = tx.selection
    let cell = tx.createElement('cell')
    let sourceCode = tx.createElement('source-code').attr('language', 'mini')
    let output = tx.createElement('output').attr('language', 'json')
    cell.append(
      sourceCode,
      output
    )
    tx.insertBlockNode(cell)
    tx.setSelection({
      type: 'property',
      path: sourceCode.getPath(),
      startOffset: 0,
      surfaceId: sel.surfaceId,
      containerId: sel.containerId
    })
  }, { action: 'insertCell' })
}

export function insertReproFig(editorSession) {
  editorSession.transaction(tx => {
    let sel = tx.selection
    let cell = tx.createElement('cell')
    let sourceCode = tx.createElement('source-code').attr('language', 'mini')
    let output = tx.createElement('output').attr('language', 'json')
    cell.append(
      sourceCode,
      output
    )
    let fig = tx.createElement('repro-fig')
    fig.append(
      tx.createElement('object-id').text(fig.id).attr({'pub-id-type': 'doi'}),
      tx.createElement('title'),
      tx.createElement('caption').append(
        tx.createElement('p')
      ),
      cell
    )
    tx.insertBlockNode(fig)
    tx.setSelection({
      type: 'property',
      path: sourceCode.getPath(),
      startOffset: 0,
      surfaceId: sel.surfaceId,
      containerId: sel.containerId
    })
  }, { action: 'insertReproFig' })
}
