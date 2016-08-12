'use strict';

var Configurator = require('substance/util/Configurator');

/**
 * A "configurator" for loading individual packages during testing.
 * 
 * @class      TestConfigurator (name)
 */
function TestConfigurator(packages) {
  TestConfigurator.super.apply(this, arguments);

  packages.forEach(function(packag) {
    this.import(packag);
  }.bind(this));
}

TestConfigurator.Prototype = function() {
};

Configurator.extend(TestConfigurator);

module.exports = TestConfigurator;
