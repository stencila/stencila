import TextBlockComponent from 'substance/ui/TextBlockComponent'

class TitleComponent extends TextBlockComponent {

  render ($$) {
    return super.render($$).addClass('sc-title')
  }

}

export default TitleComponent
