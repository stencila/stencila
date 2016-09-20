'use strict';

import TextBlock from 'substance/model/TextBlock'

function Summary () {
  Summary.super.apply(this, arguments);
}

TextBlock.extend(Summary);

Summary.type = 'summary';

module.exports = Summary;
