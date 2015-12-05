'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var $ = window.$ = require('substance/util/jquery');

var Sheet = require('./model/Sheet');

function App() {
	Component.apply(this, arguments);
}

App.Prototype = function() {

  this.render = function() {
    var el = $$('div');
    return el;
  };

};

oo.inherit(App, Component);

$(function() {
  window.app = Component.mount($$(App), $('#content'));
});
