/*
  This is used as an entry file to generate vendor/brace.js using browserify
*/
var  ace = require('brace')

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

module.exports = ace