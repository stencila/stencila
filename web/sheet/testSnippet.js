var testSnippet = {
	"name": "sum",
	"summary": "Returns the sum of values.",
	"notes": [
		"If only a single number for `values` is supplied, `sum` returns `values`"
	],
	"parameters": [
		{
			"name": "value1",
	    "descr": "The first number or range to sum up."
		},
	  {
	  	"shape": ["one", "block"],
	    "name": "value2",
	    "descr": "Additional numbers or ranges to add to value1",
	    "variadic": true,
	    "optional": true
	  }
	],
	"examples": [
		"sum(A1)",
		"sum(A1:A10, 100)",
		"sum(A1:G10, A2:G12, 14)"
	],
	"see": [
		"sumsq",
		"sumif",
		"product"
	],
	"languages": [
		"r",
		"py"
	]
}


module.exports = testSnippet;