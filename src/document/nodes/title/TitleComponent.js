import { TextBlockComponent } from 'substance'

class TitleComponent extends TextBlockComponent {

  render ($$) {
    return super.render($$).addClass('sc-title')
  }

}

export default TitleComponent
