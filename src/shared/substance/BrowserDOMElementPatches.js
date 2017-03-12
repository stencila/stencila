import {BrowserDOMElement} from 'substance'

// a proposal for custom events
BrowserDOMElement.prototype.emit = function(name, data) {
  var event = new window.Event(name, data)
  this.el.dispatchEvent(event)
}