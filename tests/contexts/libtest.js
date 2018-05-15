import { forEach, isArray } from 'substance'

const add = {
  "type": "function",
  "name": "add",
  "description": "Returns the addition of two values. The plus sign, `+`, is used as an alias for `add` e.g. `x + y` is equivalent to `add(x, y)`.\n Both values have to be of the same type: number, string, array, table or object. If the values are array, table or object, the function\n will use either extend or append to add the values.",
  "summary": "Addition of two values.",
  "methods": {
    "add(value: number|string|array|table|object, other: number|string|array|table|object)": {
      "signature": "add(value: number|string|array|table|object, other: number|string|array|table|object)",
      "params": [
        {
          "name": "value",
          "type": "number|string|array|table|object",
          "description": "The value to have other value added."
        },
        {
          "name": "other",
          "type": "number|string|array|table|object",
          "description": "The other value."
        }
      ],
      "examples": [
        {
          "usage": "// returns 6\n add(2, 4)"
        },
        {
          "usage": "// returns [1, 2, 3, 4]\n add([1,2], [3,4])"
        }
      ]
    }
  },
  "source": {
    "type": "text",
    "lang": "js",
    "data": "import append from './append'\nimport extend from './extend'\nimport is_array from './is_array'\nimport is_number from './is_number'\nimport is_object from './is_object'\nimport is_string from './is_string'\nimport is_table from './is_table'\nimport type from './type'\n\n/**\n* @summary Addition of two values.\n*\n* @description\n*\n* Returns the addition of two values. The plus sign, `+`, is used as an alias for `add` e.g. `x + y` is equivalent to `add(x, y)`.\n* Both values have to be of the same type: number, string, array, table or object. If the values are array, table or object, the function\n* will use either extend or append to add the values.\n*\n* @param {number|string|array|table|object} value The value to have other value added.\n* @param {number|string|array|table|object} other The other value.\n* @returns {number|string|array|table|object} Result of addition.\n*\n* @example\n* // returns 6\n* add(2, 4)\n*\n* @example\n* // returns [1, 2, 3, 4]\n* add([1,2], [3,4])\n*/\n\nexport default function add (value, other) {\n  if (is_number(value) && is_number(other)) return value + other\n  if (is_string(value) && is_string(other)) return value + other\n  if (is_array(value)) return append(value, other)\n  if (is_object(value) && is_object(other)) return extend(value, other)\n  if (is_table(value) && is_table(other)) return append(value, other)\n  throw new Error(`cannot add a \"${type(value)}\" and a \"${type(other)}\"`)\n}\n"
  },
  body: function add(a,b){ return a + b }
}

const multiply = {
  "type": "function",
  "name": "multiply",
  "description": "Multiply two numbers. The asterisk, `*`, is used as an alias for `multiply`\n e.g. `x * y` is equivalent to `multiply(x, y)`.",
  "title": "multiply",
  "summary": "Multiply two numbers",
  "methods": {
    "multiply(value: number, other: number): number": {
      "signature": "multiply(value: number, other: number): number",
      "params": [
        {
          "name": "value",
          "type": "number",
          "description": "The value to be multiplied."
        },
        {
          "name": "other",
          "type": "number",
          "description": "The multiplier."
        }
      ],
      "return": {
        "type": "number",
        "description": "Muliplication result."
      },
      "examples": [
        {
          "usage": "multiply(x, y)"
        },
        {
          "usage": "// return 6\n 2 * 3",
          "caption": "Example usage of multiply function."
        }
      ]
    }
  },
  "source": {
    "type": "text",
    "lang": "js",
    "data": "import assert from './assert'\nimport is_number from './is_number'\n\n/**\n* @title multiply\n* @name multiply\n*\n* @summary Multiply two numbers\n*\n* @description\n*\n* Multiply two numbers. The asterisk, `*`, is used as an alias for `multiply`\n* e.g. `x * y` is equivalent to `multiply(x, y)`.\n*\n* @param {number} value The value to be multiplied.\n* @param {number} other The multiplier.\n* @return {number} Muliplication result.\n*\n* @example multiply(x, y)\n* @example <caption>Example usage of multiply function.</caption>\n* // return 6\n* 2 * 3\n*\n*\n* @implem js\n* @author Nokome Bentley\n*/\n\nexport default function multiply (value, other) {\n  assert(is_number(value), 'parameter `value` must be a number')\n  assert(is_number(other), 'parameter `other` must be a number')\n  return value * other\n}\n"
  },
  body: function multiply(a, b) { return a * b }
}

function _sum(...vals) {
  return vals.reduce((a,b) => {
    if (b.type === 'table') {
      forEach(b.data, (vals) => {
        a += _sum(vals)
      })
      return a
    } else if (isArray(b)) {
      return _sum(...b)
    } else if (isArray(b.data)) {
      return _sum(...b.data)
    } else {
      return a+b
    }
  }, 0)
}

const sum = {
  "type": "function",
  "name": "sum",
  "description": "Sum of numbers.",
  "summary": "Sum of numbers",
  "methods": {
    "sum(values: any): number": {
      "signature": "sum(values: any): number",
      "params": [
        {
          "name": "values",
          "type": "any",
          "repeats": true,
          "description": "Numbers to sum."
        }
      ],
      "return": {
        "type": "number",
        "description": "The sum of the numbers."
      },
      "examples": [
        {
          "usage": "// returns 6\n sum([1, 2, 3])"
        },
        {
          "usage": "// returns 6\n sum(1, 2, 3)"
        }
      ]
    }
  },
  "source": {
    "type": "text",
    "lang": "js",
    "data": ""
  },
  body: _sum
}

let RAND_COUNT = 1
const rand = {
  "type": "function",
  "name": "rand",
  "description": "",
  "title": "rand",
  "summary": "Creates a random number.",
  "methods": {
    "rand(): number": {
      "signature": "rand(): number",
      "examples": [
        {
          "usage": "// returns true\n is_object([])",
          "caption": "Example usage of is_object function."
        }
      ],
      "return": {
        "type": "number",
        "description": "The result."
      },
    }
  },
  "source": {
    "type": "text",
    "lang": "js",
    "data": ""
  },
  body: function () { return RAND_COUNT++ }
}

// used in tests to reset the pseudo random generator
export function _reset_rand() {
  RAND_COUNT = 1
}

const no_params = {
  "type": "function",
  "name": "no_params",
  "description": "",
  "title": "no_params",
  "summary": "Has no params.",
  "methods": {
    "no_params(): integer": {
      "signature": "no_params(): integer",
      "examples": [],
      "return": {
        "type": "integer",
        "description": "The result."
      },
    }
  },
  "source": {
    "type": "text",
    "lang": "js",
    "data": ""
  },
  body: function () { return 5 }
}


const one_param = {
  "type": "function",
  "name": "one_param",
  "description": "",
  "title": "one_param",
  "summary": "Has one param.",
  "methods": {
    "one_param(x: number): number": {
      "signature": "one_params(x: number): number",
      "examples": [],
      "params": [
        {
          "name": "x",
          "type": "number",
          "description": "x."
        }
      ],
      "return": {
        "type": "number",
        "description": "The result."
      },
    }
  },
  "source": {
    "type": "text",
    "lang": "js",
    "data": ""
  },
  body: function (x) { return x * 1.1 }
}

const one_param_with_default = {
  "type": "function",
  "name": "one_param_with_default",
  "description": "",
  "title": "one_param_with_default",
  "summary": "Has one param with default value.",
  "methods": {
    "one_param_with_default(s: string): string": {
      "signature": "one_param_with_default(s: string): string",
      "examples": [],
      "params": [
        {
          "name": "s",
          "type": "string",
          "description": "s",
          "default": "Hello!"
        }
      ],
      "return": {
        "type": "string",
        "description": "The result."
      },
    }
  },
  "source": {
    "type": "text",
    "lang": "js",
    "data": ""
  },
  body: function (param1='Hello!') { return param1 }
}

export const libtest = {
  type: 'library',
  name: 'test',
  funcs: {
    add,
    sum,
    multiply,
    rand,
    no_params,
    one_param,
    one_param_with_default
  }
}
