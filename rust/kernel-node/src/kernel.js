#!/usr/bin/env node

// During development it can be useful to run this kernel script directly e.g.
//
//     DEV=true node rust/kernel-node/src/kernel.js
//
// Use Ctrl+D to quit, since Ctrl+C is trapped for interrupting kernel tasks.

const readline = require("readline");
const vm = require("vm");

const dev = process.env.DEV !== undefined;

const READY = dev ? "READY" : "\u{10ACDC}";
const LINE = dev ? "|" : new RegExp("\u{10ABBA}", "g");
const EXEC = dev ? "EXEC" : "\u{10B522}";
const EVAL = dev ? "EVAL" : "\u{1010CC}";
const FORK = dev ? "FORK" : "\u{10DE70}";
const LIST = dev ? "LIST" : "\u{10C155}";
const GET = dev ? "GET" : "\u{10A51A}";
const SET = dev ? "SET" : "\u{107070}";
const REMOVE = dev ? "REMOVE" : "\u{10C41C}";
const END = dev ? "END" : "\u{10CB40}";

const { stdin, stdout, stderr } = process;

// Override console log to write JSON to stdout with each arg
// treated as a separate output
console.log = function (...args) {
  for (const arg of args) {
    stdout.write(`${JSON.stringify(arg)}${END}\n`);
  }
};

// Override other console methods to output structure messages
console.debug = (message) =>
  stderr.write(
    `{"type":"ExecutionError","errorType":"Debug","errorMessage":"${message}"}${END}\n`
  );
console.info = (message) =>
  stderr.write(
    `{"type":"ExecutionError","errorType":"Info","errorMessage":"${message}"}${END}\n`
  );
console.warn = (message) =>
  stderr.write(
    `{"type":"ExecutionError","errorType":"Warning","errorMessage":"${message}"}${END}\n`
  );
console.error = (message) =>
  stderr.write(
    `{"type":"ExecutionError","errorType":"Error","errorMessage":"${message}"}${END}\n`
  );

// The execution context
const context = {
  console,
};
vm.createContext(context);

// SIGINT is handled by `vm.runInContext` but in case SIGINT is received just after a
// task finishes, or for some other reason inside the main loop, ignore it here
process.on("SIGINT", () => ({}));

const LET_REGEX = /^let\s+([\w_]+)\s*=/;
const CONST_REGEX = /^const\s+([\w_]+)\s*=/;
const VAR_REGEX = /^var\s+([\w_]+)\s*=/;
const ASSIGN_REGEX = /^\s*[\w_]+\s*=/;

// Determine if a variable is defined in the context
// This needs to be done for `let` and `const` variables
// because they do not get set on the context object
function isDefined(name) {
  try {
    vm.runInContext(name, context);
  } catch (error) {
    return false;
  }
  return true;
}

// Execute lines of code
function exec(lines) {
  // Turn any re-declarations of variables at the top level into assignments
  // (replace with spaces to retain positions for errors and stacktraces)
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];

    const letMatch = LET_REGEX.exec(line);
    if (letMatch && isDefined(letMatch[1])) {
      lines[index] = line.replace("let", "   ");
      continue;
    }

    const constMatch = CONST_REGEX.exec(line);
    if (constMatch && isDefined(constMatch[1])) {
      lines[index] = line.replace("const", "     ");
      continue;
    }

    const varMatch = VAR_REGEX.exec(line);
    if (varMatch && context[varMatch[1]] !== undefined) {
      lines[index] = line.replace("var", "   ");
      continue;
    }
  }

  // Ignore the output if associated with assignment on the last line
  let lastLineIsAssignment = false;
  if (lines.length > 0 && ASSIGN_REGEX.test(lines[lines.length - 1])) {
    lastLineIsAssignment = true;
  }

  const code = lines.join("\n");

  let output = vm.runInContext(code, context, { breakOnSigint: true });
  if (output !== undefined && !lastLineIsAssignment) {
    stdout.write(JSON.stringify(output));
  }
}

// Evaluate an expression
function eval(expression) {
  const value = vm.runInContext(expression, context, { breakOnSigint: true });
  stdout.write(JSON.stringify(value));
}

// List variables in the context
function list() {
  for (const [name, value] of Object.entries(context)) {
    let nativeType;
    if (value === null) nativeType = "null";
    else if (Array.isArray(value)) nativeType = "Array";
    else nativeType = typeof value;

    const [nodeType, valueHint] = (() => {
      switch (nativeType) {
        case "undefined":
        case "null":
          return ["Null", undefined];
        case "boolean":
          return ["Boolean", value];
        case "number":
          return ["Number", value];
        case "bigint":
          return ["Integer", value];
        case "string":
          return ["String", value.length];
        case "object":
          return ["Object", value.length];
        case "Array":
          return ["Array", value.length];
        default:
          return ["Object", undefined];
      }
    })();

    const variable = {
      type: "Variable",
      name,
      programmingLanguage: "JavaScript",
      nativeType,
      nodeType,
      valueHint,
    };

    stdout.write(`${JSON.stringify(variable)}${END}\n`);
  }
}

// Get a variable
function get(name) {
  const value = context[name];
  if (value !== undefined) {
    stdout.write(JSON.stringify(value));
  }
}

// Set a variable
function set(name, json) {
  context[name] = JSON.parse(json);
}

// Remove a variable
function remove(name) {
  delete context[name];
}

// Read lines and handle tasks
const rl = readline.createInterface({
  input: stdin,
  prompt: "",
  terminal: false,
});
rl.on("line", (task) => {
  const lines = task.split(LINE);

  try {
    (() => {
      switch (lines[0]) {
        case EXEC:
          return exec(lines.slice(1));
        case EVAL:
          return eval(lines[1]);
        case LIST:
          return list();
        case GET:
          return get(lines[1]);
        case SET:
          return set(lines[1], lines[2]);
        case REMOVE:
          return remove(lines[1]);
        default:
          throw new Error(`Unrecognized task ${lines[0]}`)
      }
    })();
  } catch (error) {
    if (error?.message === "Script execution was interrupted by `SIGINT`") {
      // Ignore error generated when interrupted
    } else {
      const msg = { type: "ExecutionError" };
      if (error.name) msg.errorType = error.name;
      if (error.message) msg.errorMessage = error.message;
      else msg.errorMessage = error.toString();
      if (error.stack) msg.stackTrace = error.stack;

      stderr.write(JSON.stringify(msg));
    }
  }

  // Indicate ready for next task
  stdout.write(`${READY}\n`);
  stderr.write(`${READY}\n`);
});

// Indicate ready for first task
stdout.write(`${READY}\n`);
stderr.write(`${READY}\n`);
