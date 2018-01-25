Prism.languages.mini = {
	// The following are based on the Prism JSON tokenizer
	'property': /"(?:\\.|[^\\"\r\n])*"(?=\s*:)/i,
	'string': {
		pattern: /"(?:\\.|[^\\"\r\n])*"|'(?:\\.|[^\\'\r\n])*'/,
		greedy: true
	},
	'number': /\b-?(?:0x[\dA-Fa-f]+|\d*\.?\d+(?:[Ee][+-]?\d+)?)\b/,
	'punctuation': /[{}[\]);,]/,
	'operator': /:/g,
	'boolean': /\b(?:true|false)\b/i,
	'null': /\bnull\b/i,
	// Additional tokens for Mini
	'function': /\b[\w_]+(?=\s*\()/,
	'symbol': /\.[\w_]+\b/,
	'variable': /\b[\w_]+(?!\s*:)/
};
