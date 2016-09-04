'use strict';

var TextBlock = require('substance/model/TextBlock');

function Title () {
  Title.super.apply(this, arguments);
}

TextBlock.extend(Title);

Title.type = 'title';

module.exports = Title;
