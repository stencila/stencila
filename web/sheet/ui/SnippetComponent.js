'use strict';

var pluck = require('lodash/collection/pluck');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

function Signature() {
  Signature.super.apply(this, arguments);
}

Signature.Prototype = function() {
  this.render = function() {
    var snippet = this.props.snippet;
    var params = pluck(snippet.parameters, 'name').join(', ');

    return $$('div').addClass('se-signature').append(
      $$('span').addClass('se-name').append(snippet.name),
      '(',
      $$('span').append(params),
      ')'
    );
  };
};

Component.extend(Signature);


function SnippetComponent() {
  SnippetComponent.super.apply(this, arguments);
}

SnippetComponent.Prototype = function() {
  this.render = function() {
    var snippet = this.props.snippet;
    var el = $$('div').addClass('sc-snippet');

    // Parameter description
    var paramsEl = $$('table').addClass('se-parameters');

    snippet.parameters.forEach(function(param) {
      paramsEl.append(
        $$('tr').append(
          $$('td').addClass('se-param-name').append(param.name),
          $$('td').addClass('se-param-descr').append(param.descr)
        )
      );
    }.bind(this));

    // Documentation
    var docEl = $$('div').addClass('se-documentation');
    docEl.append(
      $$(Signature, {snippet: snippet}),
      paramsEl,
      $$('div').addClass('se-summary').append(snippet.summary)
    );

    el.append(docEl);

    // Example
    el.append(
      $$('div').addClass('se-example').append(
        $$('div').addClass('se-label').append('Example'),
        // Display first example
        $$('div').addClass('se-example-code').append(snippet.examples[0])
      )
    );
    return el;
  };
};

Component.extend(SnippetComponent);

module.exports = SnippetComponent;
