import TextBlock from 'substance/model/TextBlock'

function Summary () {
  Summary.super.apply(this, arguments)
}

TextBlock.extend(Summary)

Summary.type = 'summary'

export default Summary
