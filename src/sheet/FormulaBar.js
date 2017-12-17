import { BodyScrollPanePackage } from 'substance'
import FormulaEditor from './FormulaEditor'

const { BodyScrollPane } = BodyScrollPanePackage

export default class FormulaBar extends FormulaEditor {

  render($$) {
    let el = $$('div').addClass('sc-formula-bar').append(
      $$('div').addClass('se-function-icon').append(
        $$('em').append(
          'ƒ',
          $$('sub').append('x')
        )
      ),
      this._renderCodeEditor($$, 'formula-bar')
    )
    return el
  }

  _renderScrollPane($$) {
    return $$(BodyScrollPane)
  }

}
