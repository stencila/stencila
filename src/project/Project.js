import { Component } from 'substance'
import { EditorPackage as TextureEditorPackage } from 'substance-texture'
import SheetEditor from '../sheet/SheetEditor'

export default class Project extends Component {

  constructor(parent, props) {
    super(parent, props)
  }

  getChildContext() {
    return {
      functionManager: this.props.functionManager,
      cellEngine: this.props.engine,
      // TODO: use a more specific name (e.g. pubMetaDb)
      pubMetaDbSession: this._getPubMetaDbSession()
    }
  }

  _getPubMetaDbSession() {
    return this.documentContainer.getEditorSession('pub-meta')
  }

  getInitialState() {
    let dc = this.props.documentContainer
    let activeDocument = dc.getDocumentEntries()[0]

    return {
      id: activeDocument.id,
      type: activeDocument.type
    }
  }

  _getDocumentContainer() {
    return this.props.documentContainer
  }

  render($$) {
    let el = $$('div').addClass('sc-project')
    let activeDocumentType = this.state.type
    let activeDocumentId = this.state.type
    let dc = this._getDocumentContainer()
    let editorSession = dc.getEditorSession(activeDocumentId)

    el.append(
      $$('div').addClass('se-header').append(
        $$('input')
          .attr('type', 'text')
          .attr('placeholder', 'Untitled Publication'),
        ...this._renderTabs($$),
        $$('button').append('+')
      )
    )

    if (activeDocumentType === 'manuscript') {
      el.append(
        $$(TextureEditorPackage.Editor, {
          editorSession
        })
      )
    } else if (activeDocumentType === 'sheet') {
      el.append(
        $$(SheetEditor, {
          editorSession
        })
      )
    }
    return el
  }


  /*
    Tabs to select one document of the publication
  */
  _renderTabs($$) {
    let tabs = []
    let documents = this.state.manifest.getDocuments()
    documents.forEach((doc) => {
      let button = $$('button').append(doc.name)
        .on('click', this._openDocument.bind(this, doc))
      if (this.state.activeDocument.path === doc.path) {
        button.addClass('sm-active')
      }
      tabs.push(button)
    })
    return tabs
  }
}
