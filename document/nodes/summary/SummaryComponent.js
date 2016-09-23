import TextBlockComponent from 'substance/ui/TextBlockComponent'

class SummaryComponent extends TextBlockComponent {

  render ($$) {
    return super.render($$).addClass('sc-summary')
  }

}


export default SummaryComponent
