import { InsertNodeCommand as SubstanceInsertNodeCommand } from 'substance'

export default class InsertReproFigCommand extends SubstanceInsertNodeCommand {

  execute(params, context) {
    var state = params.commandState
    if (state.disabled) return
    let editorSession = this._getEditorSession(params, context)
    editorSession.transaction((tx) => {
      let node = this.createNode(tx, params, context)
      tx.insertBlockNode(node)
      this.setSelection(tx, node)
    })
  }

  createNode(tx) {
    let cell = tx.createElement('cell')
    cell.append(
      tx.createElement('source-code').attr({'language': 'mini'}),
      tx.createElement('output').attr({'language': 'json'})
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
    return fig
  }
}
