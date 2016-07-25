'use strict';

/**
 * Stencila Document entry point
 * 
 * Initialises the `DocumentApp` from the content on the page.
 * Any uncaught exceptions result in a fallback to the original
 * content 
 */
window.onload = function() {

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
      html: html
    }, document.body);
  //} catch (error) {
  //  content.style.display = 'block';
  //  console.error(error);
  //}

};
