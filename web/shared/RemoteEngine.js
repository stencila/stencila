var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');

var WebsocketConnection = require('./WebsocketConnection');

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

  this.rights = {};
  this.session = {};
  this.active = false;
  this.websocket = null;
};

RemoteEngine.Prototype = function() {

  this.boot = function(cb) {
    this._request('PUT', 'boot', null, function(err, result) {
      if (err) return cb(err);
      this._activated(result);
      if(cb) cb(null, result);
    }.bind(this));
  };

  this.activate = function(cb) {
    if (!this.active) {
      this._request('PUT', 'activate', null, function(err, result) {
        if (err) return cb(err);
        this._activated(result);
        if (cb) cb(null, result);
      }.bind(this));
    }
  };

  this.deactivate = function() {
    if (this.active) {
      this._request('PUT', 'deactivate', null, function(err, result) {
        if (err) {
          console.error(err);
        } else {
          this.active = false;
          clearInterval(this.pingInterval);
        }
      }.bind(this));
    }
  };

  this.ping = function() {
    this._request('PUT', 'ping');
  };

  this.save = function(cb) {
    if (this.session.local) {
      this._call('write', [], cb);
    } else {
      this._call('store', [], cb);
    }
  }

  this.commit = function(message, cb) {
    this._call('commit',[message], cb);
  }

  /**
   * Shutdown the remote component instance
   */
  this.shutdown = function() {
    // Don't do anything for local sessions but if remote then do
    // a `store()`
    if (!this.session.local) {
      if(this.websocket) {
        this.websocket.call(this.address+'@store', [], function(result) {
        });
      } else {
        $.ajax({
          type: 'PUT',
          url: this.protocol+'//'+this.host+':'+this.port+'/'+this.address+'@store',
          // async is used so that request has time to be sent
          // See http://stackoverflow.com/questions/1821625/ajax-request-with-jquery-on-page-unload
          async: false
        });
      }
    }
  }

  // Private, local, methods

  this._call = function(name, args, cb) {
    args = args || [];
    if(this.websocket) {
      this.websocket.call(this.address+'@'+name, args, function(result) {
        if (cb) {
          cb(null, result);
        }
      });
    } else {
      this._request('PUT', name, args, function(err, result) {
        if (cb) {
          if (err) return cb(err);
          cb(null, result);
        }
      });
    }
  }

  this._request = function(method, endpoint, data, cb) {
    var self = this;
    var ajaxOpts = {
      type: method,
      url: this.protocol+'//'+this.host+':'+this.port+'/'+this.address+'@'+endpoint,
      headers: {
          'Content-Type': 'application/json; charset=utf-8',
          'Accept' : 'application/json; charset=utf-8'
      },
      // Type of data expected back
      // "json": Evaluates the response as JSON and returns a JavaScript object.
      dataType: "json",
      success: function(data) {
        if (cb) cb(null, data);
      },
      error: function(err) {
        console.error(err);
        if(err.status==401){
          $.get('/me/signin-dialog').done(function(data){
            self._dialog(data);
          });
        } else {
          if (cb) {
            try {
              cb(JSON.parse(err.responseText));
            } catch(err) {
              cb(err.responseText);
            }
          }
        }
      }
    };
    if (data) {
      ajaxOpts.data = JSON.stringify(data);
    }
    $.ajax(ajaxOpts);
  };

  /**
   * Called when engine has been activated
   *
   * @param details Activation details
   */
  this._activated = function(details) {
    this.rights = details.rights;
    if (details.session) {
      this.session = details.session;
      this.active = true;
      // Open a websocket connection to be used for
      // certain remote method calls
      if (this.session.websocket) {
        this.websocket = new WebsocketConnection(this.session.websocket);
      }
      // Begin pinging if not on localhost or localfile so that
      // session is kept alive
      if(!this.session.local) {
        this.pingInterval = setInterval(function(){
          this.ping();
        }.bind(this), 3*60*1000);
      }
    }
  };

  this._dialog = function(content) {
    var div = $('<div>')
      .appendTo($('body'))
      .html(content);
  };

};

oo.initClass(RemoteEngine);

module.exports = RemoteEngine;
