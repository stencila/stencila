'use strict';

var he = require('he');
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
  // Get `address` and `clone` from the path
  var path = window.location.pathname;
  var matches = path.match(/([^\@]+)(\@(\w+))?/);
  var address = matches[1];
  var clone = matches[3];
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

  if (!statico) {

    // Get document data as HTML content or a JSON snapshot
    // for rerendering by the `DocumentApp` and then hide content element (if any)
    var format = null;
    var data = null;
    var content = document.getElementById('content');
    if (content) {
      format = 'html';
      data = content.innerHTML;
      content.style.display = 'none';
    } else {
      var snapshot = document.getElementById('snapshot');
      if (snapshot) { 
        format = 'snapshot';
        data = JSON.parse(he.decode(snapshot.textContent || snapshot.innerHTML));
      } else {
        console.error('Neither #content or #snapshot is available to initialize the document');
      }
    }

    // Mount application on page and fallback to 
    // display orginal content on any error
    //try {
      var DocumentApp = require('./DocumentApp');
      window.app = DocumentApp.mount({
        address: address,
        clone: clone,
        format: format,
        data: data,
        local: local !== '0',
        view: view,
        reveal: reveal,
        comment: comment,
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
