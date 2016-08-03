var utilities = require('../utilities');


var loadAce = function() {
  utilities.load('/get/web/ace/ace.js', function() {
    document.dispatchEvent(new Event('ace:loaded'));
  });
}

var attachAceEditor = function(el, language, content, callback) {
  // If no language defined, default to plain text
  if (typeof language !== 'string' || language === '') language = 'text';
  function _attach() {
    var editor = this.editor = window.ace.edit(el);

    // Convert language tag to ACE mode if necessary
    // If no conversion defined here will use mode = language
    var mode = {
      'cpp':  'c_cpp',
      'js':   'javascript',
      'py':   'python',
      'r':    'r',
    }[language] || language;
    editor.getSession().setMode('ace/mode/'+mode);
    editor.setTheme("ace/theme/monokai");
    editor.setFontSize(16);
    editor.setShowPrintMargin(false);
    // Add padding before first and after last lines
    editor.renderer.setScrollMargin(5,5,0,0);
    // Set the maximum number of lines for the code. When the number
    // of lines exceeds this number a vertical scroll bar appears on the right
    editor.setOption("minLines",1);
    editor.setOption("maxLines",100);
    // Prevent warning message
    editor.$blockScrolling = Infinity;
    // Make readonly as per https://github.com/ajaxorg/ace/issues/266#issuecomment-16367687
    /*
    editor.setOptions({
      readOnly: true,
      highlightActiveLine: false,
      highlightGutterLine: false,

      wrap: true,
      indentedSoftWrap: true,
    });
    editor.renderer.$cursorLayer.element.style.opacity = 0;
    editor.textInput.getElement().disabled = true;
    editor.commands.commmandKeyBinding = {};
    */
    if (content) editor.setValue(content,1);

    callback(editor);
  }
  if (window.ace) {
    _attach()
  } else {
    document.addEventListener('ace:loaded', _attach, false);
  }
}

module.exports = {
  loadAce: loadAce,
  attachAceEditor: attachAceEditor
}
