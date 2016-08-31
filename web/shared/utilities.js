'use strict';

// Dynamic loading of scripts
function load (source, callback) {

  var head = document.getElementsByTagName('head')[0];
  var script = document.createElement('script');
  script.type = 'text/javascript';
  script.src = (window.StencilaHost || '') + source;
  if (callback) script.onload = callback;
  head.appendChild(script);

};

// Utilities related to `document.location`
var location = {

  /**
   * Get the parameters of the query (`search`) part of the location
   *
   * Based on http://stackoverflow.com/a/1099670/4625911
   */
  params: function () {

    var qs = document.location.search.split('+').join(' ');

    var params = {};
    var tokens;
    var re = /[?&]?([^=]+)=([^&]*)/g;

    while (true) {

      tokens = re.exec(qs);
      if (!tokens) break;
      params[decodeURIComponent(tokens[1])] = decodeURIComponent(tokens[2]);

    }

    return params;

  }

};

module.exports = {
  load: load,
  location: location
};
