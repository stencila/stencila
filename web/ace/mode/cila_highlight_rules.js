/**
 * Defines an editing mode for the Cila language within ACE Editor.
 *
 * See https://github.com/ajaxorg/ace/wiki/Creating-or-Extending-an-Edit-Mode
 * 
 * See http://ace.c9.io/tool/mode_creator.html for a very useful "mode creator" which
 * allows you to live test a mode
 *
 * For each token/regex pair it is necessary to have an array of tokens that is the same
 * length as the number of regex groups. If you don't capture content in a group then 
 * it won't appear in the editor  i.e. capture everything!
 * 
 * It is possible to use non-capturing groups ('(?:x)') within a group
 * to reduce the total number of groups.
 * For help on Javascript regexes...
 * 
 *  https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions
 *
 * Use token name that begin with standard Ace token names see
 *
 *  https://github.com/ajaxorg/ace/wiki/Creating-or-Extending-an-Edit-Mode#common-tokens
 *
 * Not all themes implement CSS for all standard tokens. So perhaps use multiple standard names to cover 
 * bases. End token names with a specific Cila name to allow for a custom theme to be written which 
 * recognises all our different tokens.CSS classes are generated for each dot separated element 
 * in the token name. e.g the token name
 * 
 *      constant.cila-math
 * 
 * is translated into the following CSS classes
 * 
 *      ace_constant ace_cila-math
 */

define(function(require, exports, module) {
"use strict";

var oop = require("../lib/oop");

var TextHighlightRules = require("./text_highlight_rules").TextHighlightRules;
var JavaScriptHighlightRules = require("./javascript_highlight_rules").JavaScriptHighlightRules;
var PythonHighlightRules = require("./python_highlight_rules").PythonHighlightRules;
var RHighlightRules = require("./r_highlight_rules").RHighlightRules;

var CilaHighlightRules = function() {

    /**
     * Rule generators for `exec` directives
     * 
     * Keeps a track on indentation by putting current indents on the stack
     * Inspired by https://github.com/ajaxorg/ace/blob/v1.2.0/lib/ace/mode/lua_highlight_rules.js#L132-L155
     * See also https://groups.google.com/forum/#!topic/ace-discuss/nAnKlAwDewM
     */
    function execRule(language){
        return {
            onMatch: function(value, currentState, stack){
                // Get the root indentation for this embed block
                var indents;
                var re = new RegExp(this.regex);
                var indentation = re.exec(value)[1];
                if(indentation){
                    indents = indentation.length;
                } else if(typeof stack[stack.length-1]=="number"){
                    // Get indents number that was put onto the stack by
                    // previous embed block (in `execEnd`)
                    indents = stack[stack.length-1];
                } else {
                    indents = 0;
                }
                // Clean up any indentation left behind by
                // previous embed blocks
                if(typeof stack[stack.length-1]=="number"){
                    stack.shift(); // Indents number
                    stack.shift(); // "start"
                }
                // Push to front of stack the indents and the 
                // embedded laguage rules which will be used on the next line
                stack.unshift(language+'-start',indents);

                //console.log('execRule:',stack);
                return 'keyword.cila-exec';
            },
            regex: '(^\\s*)?\\b('+language+')\\b',
            // Go to the exec arguments
            next: 'exec-args'
        }; 
    }
    function execEnd(language){
        return [
            {
                // Ignore empty and blank lines.
                // They should not be checked for indentation. 
                token: 'text',
                regex: "^\\s*$"
            },{ 
                // Lines with something on them
                // Check if indentation is less than or equal to the
                // indentation of the current embedding block
                onMatch : function(value, currentState, stack){
                    var indents = value.length;
                    if(indents<=stack[1]) {
                        stack.shift(); // Indents number
                        stack.shift(); // Language start (e.g. "r-start")
                        // Because the regex for this rule consumes the start of line
                        // and indentation character we need to "tell" any following embed
                        // block what their starting indentation is. Do that by pushing to the stack 
                        stack.unshift('start',indents);
                        // Go to start state
                        this.next = "start";
                        //console.log('execEnd:',stack);
                    } else {
                        // Stay in same state
                        this.next = "";
                    }
                    return "text";
                },
                regex: "^(\\s*)(?=.+$)" 
            } 
        ];
    }

    /**
     * The highlighter rules. These follow closely the regexes and modes in the C++ `CilaParser`
     */
    this.$rules = {
        'start':[

            // Rules for things that need to be at the start of the line
            // Because a preceding exec directives will consume the start of line and indentation
            // these may not present, so make them optional i.e. (^\s*)?
        
                // Exec directives
                execRule('exec'),
                execRule('js'),
                execRule('py'),
                execRule('r'),
                execRule('pre'),
            {
                // Output (from exec directives)
                // Must be whole line
                token: ['text','keyword.cila-out'],
                regex: /(^\s*)?(out)$/,
                next: 'out'
            },{
                // Section shortcut
                token: ['text','entity.other.attribute-name.cila-section-mark','text',
                        'text.cila-section-heading'],
                regex: /(^\s*)?(>)(\s*)(.+)/,
                next: 'plain',
            },{
                // Unordered list shortcut
                token: ['text','entity.other.attribute.cila-ul','text'],
                regex: /(^\s*)?(-)(\s+)/,
                next: 'plain',
            },{
                // Ordered list shortcut
                token: ['text','entity.other.attribute.cila-ol','text'],
                regex: /(^\s*)?(\d+\.)(\s+)/,
                next: 'plain',
            },

            {
                // Element names
                token: 'constant.language.cila-tag',
                regex: '\\b('+
                            'section|nav|article|aside|address|h1|h2|h3|h4|h5|h6|p|hr|pre|blockquote|ol|ul|li|dl|dt|dd|' +
                            'figure|figcaption|div|a|em|strong|small|s|cite|q|dfn|abbr|data|time|code|var|samp|kbd|sub|sup|i|b|u|mark|ruby|' +
                            'rt|rp|bdi|bdo|span|br|wbr|ins|del|table|caption|colgroup|col|tbody|thead|tfoot|tr|td|th|' +
                            'img' +
                        ')\\b',
                // At this point want to go straight to attributes
                // But for some reason this next cause parsing errors
                //next: 'attributes'
            },

            // Ignore space, restart if a new lines, 
            // if no match move to next state
            {
                token: 'text',
                regex: /\s+/,
            },{
                token: 'text',
                regex: /$/,
                next: 'start'
            },{
                token: 'text',
                regex: /(?=.)/,
                next: 'attributes'
            }
        ],
        'attributes': [
            {
                // General attribute
                token: 'keyword.cila-attr',
                regex: /\[[^\]]+\]/
            },{
                // id attribute
                token: 'keyword.cila-id',
                regex: /#[\w-]+/
            },{
                // class attribute
                token: 'keyword.cila-class',
                regex: /\.[\w-]+/
            },

            // Ignore space, restart if a new lines, 
            // if no match move to next state
            {
                token: 'text',
                regex: /\s+/,
            },{
                token: 'text',
                regex: /$/,
                next: 'start'
            },{
                token: 'text',
                regex: /(?=.)/,
                next: 'directives'
            }
        ],
        'directives': [
            {
                // Directives with no argument
                token: ['keyword'],
                regex: /\b(each|else|default)\b/
            },{
                // Directives with a single string argument
                token: ['keyword','text','variable.parameter'],
                regex: /\b(when|macro)(\s+)([^\s}]+)/
            },{
                // Directives with a single expression argument
                token: ['keyword','text','support.function.cila-expr'],
                regex: /\b(when|with|text|if|elif|switch|case)(\s+)([^\s}]+)/
            },{
                // Directives with a single selector argument
                token: ['keyword','text','storage.type.cila-selector'],
                regex: /\b(refer)(\s+)([\.\#\w\-]+)/
            },{
                // `attr` directive
                token: ['keyword','text','variable.parameter.cila-name','text',
                        'keyword','text','support.function.cila-expr'],
                regex: /\b(attr)(\s+)([\w\-]+)(?:(\s+)(value)(\s+)([^\s}]+))?/
            },{
                // `for` directive
                token: ['keyword','text','variable.parameter.cila-name','text',
                        'keyword','text','support.function.cila-expr'],
                regex: /\b(for)(\s+)(\w+)(\s+)(in)(\s+)([^\s}]+)/
            },{
                // `include` directive
                token: ['keyword','text','variable.parameter.cila-address','text',
                        'keyword','text','storage.type.cila-selector'],
                regex: /\b(include)(\s+)([\w\./]+)(?:(\s+)(select)(\s+)([\.\#\w\-]+))?/
            },{
                // `set` directive
                token: ['keyword','text','variable.parameter.cila-name','text',
                        'keyword','text','support.function.cila-expr'],
                regex: /\b(set)(\s+)([\w]+)(\s+)(to)(\s+)([^\s}]+)/
            },{
                // `include` modifier directives
                token: ['keyword','text','storage.type.cila-selector'],
                regex: /\b(delete|replace|change|before|after|prepend|append)(\s+)([\.\#\w\-]+)/
            },{
                // `par` directive
                token: ['keyword','text','variable.parameter.cila-name','text',
                        'keyword','text','variable.parameter.cila-name','text',
                        'keyword','text','support.function'],
                regex: /\b(par)(\s+)([\w]+)(?:(\s+)(type)(\s+)([\w]+))?(?:(\s+)(value)(\s+)([^\s}]+))?/
            },{
                // `begin` and `end` directives
                token: ['keyword','text','variable.parameter'],
                regex: /\b(begin|end)( +)(\d+)/
            },{
                // `comments` directive
                token: ['keyword','text','variable.parameter'],
                regex: /(\bcomments)(?:(\s+)([\#\.\w\-]+))?/
            },{
                // `comment` directive
                token: ['keyword','text','variable.parameter',
                        'keyword','text','variable.parameter'],
                regex: /(\bcomment)(\s+)(.+?)( at)(\s+)([\w\-\:\.]+)/
            },

            // Ignore space, restart if a new lines, 
            // if no match move to next state
            {
                token: 'text',
                regex: /\s+/,
            },{
                token: 'text',
                regex: /$/,
                next: 'start'
            },{
                token: 'text',
                regex: /(?=.)/,
                next: 'flags'
            } 
        ],
        'flags': [
            {
                // Hash
                token: 'comment.cila-hash',
                regex: /&[a-zA-Z0-9]+/
            },{
                // Index
                token: 'comment.cila-index',
                regex: /\^\d+/
            },{
                // Off
                token: 'comment.cila-off',
                regex: /~off/
            },{
                // Error
                token: 'comment.cila-error',
                regex: /(!\")([^\"]+)(\")(@\d+(,\d+)?)?/
            },{
                // Warning
                token: 'keyword.cila-warning',
                regex: /(%\")([^\"]+)(\")(@\d+(,\d+)?)?/
            },

            // Ignore space, restart if a new lines, 
            // if no match move to next state
            {
                token: 'text',
                regex: /\s+/,
            },{
                token: 'text',
                regex: /$/,
                next: 'start'
            },{
                token: 'text',
                regex: /(?=.)/,
                next: 'plain'
            } 
        ],
        'plain': [
            {
                // Open brace, into "start" state again
                token: 'comment.cila-brace-open',
                regex: /{/,
                next: 'start'
            },{
                // Close brace, remain in this state
                token: 'comment.cila-brace-close',
                regex: /}/,
            },{
                // Hyperlink shorthand
                token: ['string.quoted','string','string.quoted','string','string.quoted'],
                regex: /(\[)([^\]]*)(\]\()([^\)]+)(\))/
            },{
                // Refer
                token: 'string',
                regex: /@[\.\#\w\-]+/
            },{
                // Auto-hyperlink
                token: 'string',
                regex: /\bhttp(s)?:\/\/[^\s]+\b/
            },{
                // Begin & end markers
                token: 'comment.cila-begin-end',
                regex: /(\[[\w-]+\]\>)|(\<\[[\w-]+\])/
            },
            
            {
                // Enter "empha" state
                token: 'comment.cila-mark',
                regex: /_/,
                next: 'empha'
            },{
                // Enter "strong" state
                token: 'comment.cila-mark',
                regex: /\*/,
                next: 'strong'
            },{
                // Enter "code" state
                token: 'comment.cila-mark',
                regex: /`/,
                next: 'code'
            },{
                // Enter "asciimath" state
                token: 'comment.cila-mark',
                regex: /\|/,
                next: 'asciimath'
            },{
                // Enter "tex" state
                token: 'comment.cila-mark',
                regex: /\\\(/,
                next: 'tex'
            },


            {
                // End of line, return to "start" state
                token: 'text',
                regex: /$/,
                next: 'start'
            }
        ],

        'exec-args':[
            {
                // Format argument
                token: ['keyword','text','variable.parameter'],
                regex: /\b(format)(\s+)([^\s]+)/
            },{
                // Size argument
                token: ['keyword','text','variable.parameter'],
                regex: /\b(size)(\s+)([^\s]+)/
            },{
                // Const argument
                token: ['keyword'],
                regex: /\b(const)/
            },{
                // Show argument
                token: ['keyword'],
                regex: /\b(show)/
            },{
                // Hash flag
                token: 'comment.cila-hash',
                regex: /&[a-zA-Z0-9]+/
            },{
                // Error flag
                token: 'comment.cila-error',
                regex: /(!\")([^\"]+)(\")(@\d+(,\d+)?)?/
            },{
                // Warning flag
                token: 'keyword.cila-warning',
                regex: /(%\")([^\"]+)(\")(@\d+(,\d+)?)?/
            },
            { 
                // End of line so move to the next state in the stack
                // This was not necessary on the ACE mode creator but is in production (?)
                onMatch : function(value, currentState, stack){
                    this.next = stack[0];
                },
                regex: /$/ 
            } 
        ],

        'out':[
            {
                // Empty line, go to "start" state
                token: 'text',
                regex: /^$/,
                next: 'start'
            },{
                defaultToken: 'comment.cila-out-content',
            }                 
        ],

        'empha':[
            {
                // Exit "empha" state
                token: 'comment.cila-mark',
                regex: /_/,
                next: 'plain'
            },{
                defaultToken: 'string.quoted.cila-empha',
            }                 
        ],
        'strong':[
            {
                // Exit "strong" state
                token: 'comment.cila-mark',
                regex: /\*/,
                next: 'plain'
            },{
                defaultToken: 'string.quoted.cila-strong',
            }                 
        ],
        'code':[
            {
                // Exit "code" state
                token: 'comment.cila-mark',
                regex: /`/,
                next: 'plain'
            },{
                defaultToken: 'string.quoted.cila-code',
            }                 
        ],
        'asciimath':[
            {
                // Exit "asciimath" state
                token: 'comment.cila-mark',
                regex: /\|/,
                next: 'plain'
            },{
                defaultToken: 'string.quoted.cila-asciimath',
            }                 
        ],
        'tex':[
            {
                // Exit "tex" state
                token: 'comment.cila-mark',
                regex: /\\\)/,
                next: 'plain'
            },{
                defaultToken: 'string.quoted.cila-tex',
            }                 
        ]
    };

    this.embedRules(JavaScriptHighlightRules, "exec-", execEnd());
    this.embedRules(JavaScriptHighlightRules, "js-", execEnd());
    this.embedRules(PythonHighlightRules, "py-", execEnd());
    this.embedRules(RHighlightRules, "r-", execEnd());
    
    this.embedRules(TextHighlightRules, "pre-", execEnd());

    this.normalizeRules();
};
oop.inherits(CilaHighlightRules, TextHighlightRules);

exports.CilaHighlightRules = CilaHighlightRules;
});
