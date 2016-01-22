'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function SnippetComponent() {
  SnippetComponent.super.apply(this, arguments);
}

SnippetComponent.Prototype = function() {
  this.render = function() {
    var snippet = this.props.snippet;
    var el = $$('div').addClass('sc-snippet');

    // Parameter description
    var paramsEl = $$('table').addClass('se-parameters');

    snippet.parameters.forEach(function(param, i) {

      var paramEl = $$('tr').addClass('se-param').append(
        $$('td').addClass('se-param-name').append(param.name),
        $$('td').addClass('se-param-descr').append(param.descr)
      );

      if (i === this.props.paramIndex) {
        paramEl.addClass('sm-active');
      }
      paramsEl.append(paramEl);
    }.bind(this));

    // Documentation
    var docEl = $$('div').addClass('se-documentation');
    docEl.append(
      $$(SnippetComponent.Signature, {snippet: snippet, paramIndex: this.props.paramIndex}),
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

SnippetComponent.Signature = function() {
  SnippetComponent.Signature.super.apply(this, arguments);
};

SnippetComponent.Signature.Prototype = function() {
  this.render = function() {
    var snippet = this.props.snippet;

    var paramsEl = $$('span').addClass('se-signature-params');

    snippet.parameters.forEach(function(param, i) {
      var paramEl = $$('span').addClass('se-signature-param').append(param.name);

      if (i === this.props.paramIndex) {
        paramEl.addClass('sm-active');
      }

      paramsEl.append(paramEl);
      if (i < snippet.parameters.length - 1) {
        paramsEl.append(',');
      }

    }.bind(this));

    return $$('div').addClass('se-signature').append(
      $$('span').addClass('se-name').append(snippet.name),
      '(',
      $$('span').append(paramsEl),
      ')'
    );
  };
};

Component.extend(SnippetComponent.Signature);

module.exports = SnippetComponent;
