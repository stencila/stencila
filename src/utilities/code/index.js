// This module gets loaded as part of the collab code
// run in Node.js. But `brace` needs `window` so fails there. Protect from that...
let ace
if (typeof window !== 'undefined') {
  ace = require('brace')

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
}

var attachAceEditor = function (el, content, options, callback) {
  var editor = ace.edit(el)
  updateAceEditor(editor, options)
  if (content) editor.setValue(content, 1)
  if (callback) callback(editor)
}

var setAceEditorMode = function (editor, language) {
  // Convert language tag to ACE mode if necessary
  // If no language defined, default to plain text
  // If no conversion defined here will use mode = language
  if (typeof language !== 'string' || language === '') language = 'text'
  var mode = {
    'bash': 'sh',
    'cpp': 'c_cpp',
    'js': 'javascript',
    'md': 'markdown',
    'py': 'python',
    'r': 'r',
    'rb': 'ruby'
  }[language] || language
  editor.getSession().setMode('ace/mode/' + mode)
}

var updateAceEditor = function (editor, options) {
  options = options || {}

  // Stuff that is not yet actually an option
  editor.setTheme('ace/theme/monokai')
  editor.setShowPrintMargin(false)
  // Add padding before first and after last lines
  editor.renderer.setScrollMargin(5, 5, 0, 0)
  // Wrapping
  editor.setOptions({
    wrap: true,
    indentedSoftWrap: true
  })
  // Prevent warning message
  editor.$blockScrolling = Infinity

  setAceEditorMode(editor, options.language || 'text')

  editor.setFontSize(options.fontSize || 16)

  // Set the maximum number of lines for the code. When the number
  // of lines exceeds this number a vertical scroll bar appears on the right
  editor.setOption('minLines', options.minLines || 1)
  editor.setOption('maxLines', options.maxLines || Infinity)

  // TODO complete implementation of turning back on. commands etc
  if (options.readOnly) {
    // Make readonly as per https://github.com/ajaxorg/ace/issues/266#issuecomment-16367687
    editor.setOptions({
      readOnly: true,
      highlightActiveLine: false,
      highlightGutterLine: false
    })
    editor.renderer.$cursorLayer.element.style.opacity = 0
    editor.textInput.getElement().disabled = true
    editor.commands.commmandKeyBinding = {}
  } else {
    editor.setOptions({
      readOnly: false,
      highlightActiveLine: true,
      highlightGutterLine: true
    })
    editor.renderer.$cursorLayer.element.style.opacity = 1
    editor.textInput.getElement().disabled = false
  }
}

export default {
  attachAceEditor: attachAceEditor,
  setAceEditorMode: setAceEditorMode,
  updateAceEditor: updateAceEditor
}
