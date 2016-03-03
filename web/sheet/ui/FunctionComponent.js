'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;

function FunctionComponent() {
  FunctionComponent.super.apply(this, arguments);
}

FunctionComponent.Prototype = function() {

  // Lifecycle
  // ------------------------------------

  /*
    Initial state signature
  */
  this.getInitialState = function() {
    return {
      func: null // the function specification: needs to be loaded from server
    };
  };

  /*
    Triggers loading of remote data needed for display
  */
  this._init = function() {
    this._loadFunctionSpec();
  };

  /*
    On initial mount
  */
  this.didMount = function() {
    console.log('FunctionComponent.didmount');
    this._init();
  };

  /*
    When new props funcName, paramIndex are set 
  */
  this.willReceiveProps = function(newProps) {
    if (this.props.funcName !== newProps.funcName) {
      this.dispose();
      this._init();
    }
  };

  // Render
  // ------------------------------------  

  this.render = function() {
    var el = $$('div').addClass('sc-function');
    
    var func = this.state.func;  
    var funcName = this.props.funcName;

    if (func) {
      // Function signature
      var signatureEl = $$(FunctionComponent.Signature, {
        func: func, 
        paramIndex: this.props.paramIndex
      });

      // Function title
      var titleEl = $$('div').addClass('se-title').append(func.title);

      // Parameter descriptions
      var paramsEl = $$('table').addClass('se-parameters');
      func.parameters.forEach(function(param, i) {
        var paramEl = $$('tr').addClass('se-param').append(
          $$('td').addClass('se-param-name').append(param.name),
          $$('td').addClass('se-param-descr').append(param.description)
        );
        if (i === this.props.paramIndex) {
          paramEl.addClass('sm-active');
        }
        paramsEl.append(paramEl);
      }.bind(this));

      // Summary
      var summaryEl = $$('div').addClass('se-summary').append(func.summary||'');

      // Documentation
      var docEl = $$('div').addClass('se-documentation');
      docEl.append(
        titleEl,
        signatureEl,
        paramsEl,
        summaryEl
      );
      el.append(docEl);

      // Example
      var example;
      if (func.examples) {
        example = func.examples[0];
      }

      if (example) {
        el.append(
          $$('div').addClass('se-example').append(
            $$('div').addClass('se-label').append('Example'),
            // Display first example
            $$('div').addClass('se-example-code').append(example)
          )
        );        
      }
    } else {
      el.append('Loading function specification for '+ funcName);
    }

    return el;
  };

  // Utils
  // ------------------------------------

  /**
   * Load a function specification from the
   * sheet execution engine
   */
  this._loadFunctionSpec = function() {
    var engine = this.context.engine;
    var funcName = this.props.funcName;
    engine.function(funcName, function(err, func) {
      if (err) {
        console.error(funcName, 'could not be loaded');
      }
      this.setState({
        func: func
      });
    }.bind(this));
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
