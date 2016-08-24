var utilities = require('../utilities');


var loadAce = function() {
  utilities.load('/get/web/ace/ace.js', function() {
    document.dispatchEvent(new Event('ace:loaded'));
  });
}

var attachAceEditor = function(el, content, options, callback) {
  function _attach() {
    var editor = window.ace.edit(el);
    updateAceEditor(editor, options);
    if (content) editor.setValue(content,1);
    callback(editor);
  }
  if (window.ace) {
    _attach()
  } else {
    document.addEventListener('ace:loaded', _attach, false);
  }
}

var setAceEditorMode = function(editor, language) {
  // Convert language tag to ACE mode if necessary
  // If no language defined, default to plain text
  // If no conversion defined here will use mode = language
  if (typeof language !== 'string' || language === '') language = 'text';
  var mode = {
    'cpp':  'c_cpp',
    'js':   'javascript',
    'py':   'python',
    'r':    'r',
  }[language] || language;
  editor.getSession().setMode('ace/mode/' + mode);  
}

var updateAceEditor = function(editor, options) {

  // Stuff that is not yet actually an option
  editor.setTheme("ace/theme/monokai");
  editor.setShowPrintMargin(false);
  // Add padding before first and after last lines
  editor.renderer.setScrollMargin(5,5,0,0);
  // Wrapping
  editor.setOptions({
    wrap: true,
    indentedSoftWrap: true,
  });
  // Prevent warning message
  editor.$blockScrolling = Infinity;

  setAceEditorMode(editor, options.language);

  editor.setFontSize(options.fontSize || 16);

  // Set the maximum number of lines for the code. When the number
  // of lines exceeds this number a vertical scroll bar appears on the right
  editor.setOption("minLines", 1);
  editor.setOption("maxLines", options.maxLines || Infinity);

  // TODO complete implementation of turning back on. commands etc
  if (options.readOnly) {
    // Make readonly as per https://github.com/ajaxorg/ace/issues/266#issuecomment-16367687
    editor.setOptions({
      readOnly: true,
      highlightActiveLine: false,
      highlightGutterLine: false
    });
    editor.renderer.$cursorLayer.element.style.opacity = 0;
    editor.textInput.getElement().disabled = true;
    editor.commands.commmandKeyBinding = {};
  } else {
    editor.setOptions({
      readOnly: false,
      highlightActiveLine: true,
      highlightGutterLine: true
    });
    editor.renderer.$cursorLayer.element.style.opacity = 1;
    editor.textInput.getElement().disabled = false; 
  }

}

module.exports = {
  loadAce: loadAce,
  attachAceEditor: attachAceEditor,
  setAceEditorMode: setAceEditorMode,
  updateAceEditor: updateAceEditor
}
