import { Component } from 'substance'
import FunctionComponent from './FunctionComponent'
import SelectFunctionComponent from './SelectFunctionComponent'

/*
  The CellEditor is different to a regular TextPropertyEditor
  in the regard that it doesn't update the document during editing,
  only at the end.
*/
export default
class CellEditor extends Component {

  render($$) {
    var el = $$('div')
    el.append(
      $$('input')
        .attr('spellcheck', 'false')
        .attr('value', this.props.content)
        .on('keydown', this.onKeydown)
        .on('keypress', this.onKeypress)
        .on('click', this.onClick)
        .on('input', this.onChange)
        .ref('editor')
    )
    if (this.state.funcName) {
      el.append($$(FunctionComponent, {
        funcName: this.state.funcName,
        paramIndex: this.state.paramIndex
      }).ref('function')); // ref is needed so the component is not wiped on each keystroke
    } else if (this.state.suggestedFunctions) {
      // Render function name suggestor
      el.append($$(SelectFunctionComponent, {
        entries: this.state.suggestedFunctions
      }).ref('selectFunction'))
    }
    return el
  }

  didMount() {
    var el = this._getTextArea()
    el.focus()
    if (this.props.select === "all") {
      el.select()
    } else if (this.props.select === "last") {
      var l = this.props.content.length
      el.setSelectionRange(l, l)
    }
  }

  getContent() {
    return this._getTextArea().value
  }

  _getTextArea() {
    return this.el.querySelector('input')
  }

  onKeydown(event) {
    // console.log('CellEditor.onKeydown()', 'keyCode=', event.keyCode, event)
    var handled = false
    // ENTER
    if (event.keyCode === 13) {
      if (!event.ctrlKey && !event.shiftKey) {
        this.send('commitCellChange', this.getContent(), 'enter')
        handled = true
      }
    } else if (event.keyCode === 27) {
      this.send('discardCellChange')
      handled = true
    } else if (event.keyCode >= 37 && event.keyCode <= 40) {
      this._detectFunction()
    }
    if (handled) {
      event.stopPropagation()
      event.preventDefault()
    }
  }

  onKeypress(event) {
    // console.log('CellEditor.onKeypress()', 'keyCode=', event.keyCode)
    var handled = false
    if (handled) {
      event.stopPropagation()
      event.preventDefault()
    }
  }

  onClick() {
    this._detectFunction()
  }

  onChange() {
    this._detectFunction()
  }

  /*
    Iterates over available function names and matches the current input string

    TODO: @nokome: the _matcher needs to be improved! Also you may want to
          limit the number of suggested function names
  */
  _matchFunctionNames(str) {
    if (!str) return []; // don't match anything for an empty string
    var _matcher = new RegExp('\^'+regexpEscape(str), 'g')

    var matches = []
    var funcs = this._getAvailableFunctions()

    funcs.forEach(function(funcName) {
      if (_matcher.exec('='+funcName)) {
        matches.push(funcName)
      }
    })
    return matches
  }

  _getAvailableFunctions() {
    // var engine = this.context.engine
    // return engine.getFunctionList()
    return []
  }

  _detectFunction() {
    var _availableFuncs = this._getAvailableFunctions()
    var _function_re_str = '\\b(' + _availableFuncs.map(regexpEscape).join('|') + ')[(]'

    setTimeout(function() {
      var el = this._getTextArea()
      var source = el.value
      var pos = el.selectionStart
      // only if collapsed
      if (pos === el.selectionEnd) {
        source = source.slice(0, pos)
        var re = new RegExp(_function_re_str, 'gi')
        var lastMatch, match
        while ( (match = re.exec(source)) ) {
          lastMatch = match
        }

        if (lastMatch) {
          var funcName = lastMatch[1]
          var startPos = lastMatch.index+1
          var argsPos = startPos + lastMatch[0].length
          var currentArg = this._detectCurrentArg(source.slice(argsPos))
          var newState = {
            funcName: funcName,
            paramIndex: currentArg.argIdx
          }
          this.setState(newState)
        } else {
          // Check if any available function name matches partly so we can suggest it
          var suggestedFunctions = this._matchFunctionNames(source)

          if (suggestedFunctions.length > 0) {
            this.setState({
              suggestedFunctions: suggestedFunctions
            })
          } else {
            this.setState({
              funcName: false
            })
          }
        }
      }
    }.bind(this))
  }

  _detectCurrentArg(str) {
    // on each ',' increase counter
    // on '(' skip content to allow for nested expressions
    var argIdx = 0
    var stackCount = 0
    var skip = false
    for(var pos = 0; pos < str.length; pos++) {
      var c = str[pos]
      if (skip) {
        if (c === '(') {
          stackCount++
        } else if (c === ')') {
          stackCount--
        }
        if (stackCount === 0) {
          skip = false
        }
      }
      else if (c === ',') {
        argIdx++
      }
      else if (c === '(') {
        stackCount++
        skip = true
      }
    }
    return {
      argIdx: argIdx
    }
  }

}

function regexpEscape(s) {
  return s.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&')
}
