'use strict'

import TextBlock from 'substance/model/TextBlock'

function Title () {
  Title.super.apply(this, arguments)
}

TextBlock.extend(Title)

Title.type = 'title'

export default Title
