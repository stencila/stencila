import { TextBlockComponent } from 'substance'

class SummaryComponent extends TextBlockComponent {

  render ($$) {
    return super.render($$).addClass('sc-summary')
  }

}

export default SummaryComponent
