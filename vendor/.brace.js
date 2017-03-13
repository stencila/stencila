/*
  Used as an entry file to generate vendor/brace.min.js
*/
var brace = require('brace')

require('brace/mode/c_cpp')
require('brace/mode/javascript')
require('brace/mode/json')
require('brace/mode/html')
require('brace/mode/markdown')
require('brace/mode/python')
require('brace/mode/r')
require('brace/mode/ruby')
require('brace/mode/sh')
require('brace/theme/monokai')

module.exports = brace
