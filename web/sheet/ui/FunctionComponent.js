'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function FunctionComponent() {
  FunctionComponent.super.apply(this, arguments);
}

FunctionComponent.Prototype = function() {
  this.render = function() {
    var func = this.props.func;
    var el = $$('div').addClass('sc-function');

    // Parameter description
    var paramsEl = $$('table').addClass('se-parameters');

    func.parameters.forEach(function(param, i) {

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
      $$(FunctionComponent.Signature, {func: func, paramIndex: this.props.paramIndex}),
      paramsEl,
      $$('div').addClass('se-summary').append(func.summary)
    );

    el.append(docEl);

    // Example
    el.append(
      $$('div').addClass('se-example').append(
        $$('div').addClass('se-label').append('Example'),
        // Display first example
        $$('div').addClass('se-example-code').append(func.examples[0])
      )
    );
    return el;
  };
};

Component.extend(FunctionComponent);

FunctionComponent.Signature = function() {
  FunctionComponent.Signature.super.apply(this, arguments);
};

FunctionComponent.Signature.Prototype = function() {
  this.render = function() {
    var func = this.props.func;

    var paramsEl = $$('span').addClass('se-signature-params');

    func.parameters.forEach(function(param, i) {
      var paramEl = $$('span').addClass('se-signature-param').append(param.name);

      if (i === this.props.paramIndex) {
        paramEl.addClass('sm-active');
      }

      paramsEl.append(paramEl);
      if (i < func.parameters.length - 1) {
        paramsEl.append(',');
      }

    }.bind(this));

    return $$('div').addClass('se-signature').append(
      $$('span').addClass('se-name').append(func.name),
      '(',
      $$('span').append(paramsEl),
      ')'
    );
  };
};

Component.extend(FunctionComponent.Signature);

module.exports = FunctionComponent;
