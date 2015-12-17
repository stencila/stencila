var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');

var RemoteEngine = function() {

  var location = window.location;
  this.protocol = location.protocol;
  if(this.protocol==='file:') this.host = 'localfile';
  else this.host = location.hostname;
  this.port = location.port;

  // Address
  this.address = null;
  // ... from <meta> tag
  var address = $('head meta[itemprop=address]');
  if(address.length) this.address = address.attr('content');
  // ... or from url
  if(!this.address) {
    // Remove the leading /
    var path = window.location.pathname.substr(1);
    // Remove the last part of path if it is a title slug
    var lastIndex = path.lastIndexOf('/');
    var last = path.substr(lastIndex);
    if(last.substr(last.length-1)=="-") this.address = path.substr(0,lastIndex);
  }

};

RemoteEngine.Prototype = function() {

  this.request = function(method, endpoint, data, cb) {
    var ajaxOpts = {
      type: method,
      url: this.protocol+'//'+this.host+':'+this.port+'/'+this.address+'@'+endpoint,
      // Specify JSON as content type to send
      contentType: "application/json; charset=UTF-8",
      // Type of data expected back
      // "json": Evaluates the response as JSON and returns a JavaScript object.
      dataType: "json",
      success: function(data) {
        cb(null, data);
      },
      error: function(err) {
        console.error(err);
        cb(err.responseText);
      }
    };
    if (data) {
      ajaxOpts.data = JSON.stringify(data);
    }
    $.ajax(ajaxOpts);
  };

};

oo.initClass(RemoteEngine);

module.exports = RemoteEngine;
