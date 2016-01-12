var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');

var RemoteEngine = function() {

  var location = window.location;
  this.protocol = location.protocol;

  if (location.hostname==='0.0.0.0' || location.hostname==='127.0.0.1') this.host = 'localhost';
  else if (this.protocol==='file:') this.host = 'localfile';
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

  this.active = false;

};

RemoteEngine.Prototype = function() {

  this.activate = function() {
    if (!this.active) {
      this.request('PUT', 'activate', null, function(err, result) {
        if (err) { console.error(err); }
        else {
          this.active = true;
          this.pingInterval = setInterval(function(){
            this.ping();
          }.bind(this), 3*60*1000);
        }
      }.bind(this));
    }
  };

  this.deactivate = function() {
    if (this.active) {
      this.request('PUT', 'deactivate', null, function(err, result) {
        if (err) { console.error(err); }
        else {
          this.active = false;
          clearInterval(this.pingInterval);
        }
      }.bind(this));
    }
  };

  this.ping = function() {
    if (this.active) {
      this.request('PUT', 'ping', null, function(err, result) {
        if (err) { console.error(err); }
      }.bind(this));
    }
  };

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
