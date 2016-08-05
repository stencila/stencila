'use strict';

var utilities = require('../shared/utilities');
var code = require('../shared/code');

/**
 * Stencila Document entry point
 * 
 * Initialises the `DocumentApp` from the content on the page.
 * Any uncaught exceptions result in a fallback to the original
 * content 
 */
window.onload = function() {
  // Check parameters for options
  var params = utilities.location.params();
  var statico = (params.static || '0') != '0';
  var reveal = (params.reveal || '0') != '0';
  var edit = (params.edit || '0') != '0';

  if (statico) {

    // Get the document content DOM element
    var content = document.getElementById('content');

    // Get document content as HTML for rerendering by the `DocumentApp`
    // and then hide content element
    var html = '';
    if (content) {
      html = content.innerHTML;
      content.style.display = 'none';
    }

    // Mount application on page and fallback to 
    // display orginal content on any error
    //try {
      var DocumentApp = require('./DocumentApp');
      window.app = DocumentApp.mount({
        html: html,
        reveal: reveal,
        edit: edit
      }, document.body);

      // Load ACE editor
      code.loadAce();

    //} catch (error) {
    //  content.style.display = 'block';
    //  console.error(error);
    //}
  
  }

};
