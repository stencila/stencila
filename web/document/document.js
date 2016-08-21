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
  // Check URL parameters for options with defaults
  // determined by host.
  // Note: thses may be overidden in the `DocumentApp` depending
  // upon user rights for the document
  var params = utilities.location.params();
  var local = (window.host === 'localhost:7373') ? '1' : '0';
  // Static/dynamic (Javasctpit loaded or not) defaults to dynamic
  var statico = (params.static || '0') !== '0';
  // View defaults to visual
  var view = params.view || 'visual';
  // Reveal, comment and edit modes default to `on` when
  // local and `off` otherwise
  var reveal = (params.reveal || local) !== '0';
  var comment = (params.comment || local) !== '0';
  var edit = (params.edit || local) !== '0';
  // Collaboration jam session defaults to `null`
  var jam = params.jam || null;

  if (!statico) {

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
        local: local !== '0',
        view: view,
        reveal: reveal,
        comment: comment,
        edit: edit,
        jam: jam
      }, document.body);

      // Load ACE editor
      code.loadAce();

    //} catch (error) {
    //  content.style.display = 'block';
    //  console.error(error);
    //}
  
  }

};
