#!/usr/bin/env node

// During development it can be useful to run this kernel script directly e.g.
//
//     DEV=true node rust/kernel-node/src/kernel.js
//
// Use Ctrl+D to quit, since Ctrl+C is trapped for interrupting kernel tasks.

const child_process = require("child_process");
const fs = require("fs");
const os = require("os");
const readline = require("readline");
const vm = require("vm");

const dev = process.env.DEV !== undefined;

const READY = dev ? "READY" : "\u{10ACDC}";
const LINE = dev ? "|" : new RegExp("\u{10ABBA}", "g");
const EXEC = dev ? "EXEC" : "\u{10B522}";
const EVAL = dev ? "EVAL" : "\u{1010CC}";
const FORK = dev ? "FORK" : "\u{10DE70}";
const INFO = dev ? "INFO" : "\u{10EE15}";
const PKGS = dev ? "PKGS" : "\u{10BEC4}";
const LIST = dev ? "LIST" : "\u{10C155}";
const GET = dev ? "GET" : "\u{10A51A}";
const SET = dev ? "SET" : "\u{107070}";
const REMOVE = dev ? "REMOVE" : "\u{10C41C}";
const END = dev ? "END" : "\u{10CB40}";

// If IO streams are specified in args use them (for forks), otherwise
// use standard IO on the process.
let stdin;
let stdout;
let stderr;
let inheritedImports = [];
let inheritedVariables = {};
if (process.argv.length > 2) {
  stdin = fs.createReadStream(process.argv[2]);
  stdout = fs.createWriteStream(process.argv[3], { flags: "a" });
  stderr = fs.createWriteStream(process.argv[4], { flags: "a" });
  inheritedImports = JSON.parse(process.argv[5]);
  inheritedVariables = JSON.parse(process.argv[6]);
} else {
  ({ stdin, stdout, stderr } = process);
}

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
    `{"type":"ExecutionMessage","level":"Debug","message":"${message}"}${END}\n`
  );
console.info = (message) =>
  stderr.write(
    `{"type":"ExecutionMessage","level":"Info","message":"${message}"}${END}\n`
  );
console.warn = (message) =>
  stderr.write(
    `{"type":"ExecutionMessage","level":"Warning","message":"${message}"}${END}\n`
  );
console.error = (message) =>
  stderr.write(
    `{"type":"ExecutionMessage","level":"Error","message":"${message}"}${END}\n`
  );

// Create the execution context
const context = {
  ...inheritedVariables,
  require,
  console,
  process,
};
vm.createContext(context);

// Apply inherited imports ignoring any errors
for (const line of inheritedImports) {
  try {
    vm.runInContext(line, context);
  } catch {
    // Pass
  }
}

// Lines which imported modules by the name of the variable for the module.
// Used to pass these on to forks so that they have the same modules available
// to them (in addition to variables)
const imports = {};

// SIGINT is handled by `vm.runInContext` but in case SIGINT is received just after a
// task finishes, or for some other reason inside the main loop, ignore it here
process.on("SIGINT", () => ({}));

const IMPORT_REGEX =
  /^(?:(const|let|var)\s+)?([\w_]+)\s*=\s*(require|import)\(/;
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
function execute(lines) {
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];

    // Record any lines for module imports so that they can be sent to forks
    const importMatch = IMPORT_REGEX.exec(line);
    if (importMatch) {
      const name = importMatch[1];
      imports[name] = line;
    }

    // Turn any re-declarations of variables at the top level into assignments
    // (replace with spaces to retain positions for errors and stacktraces)

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

  const output = vm.runInContext(code, context, { breakOnSigint: true });
  if (output !== undefined && !lastLineIsAssignment) {
    stdout.write(JSON.stringify(output));
  }
}

// Evaluate an expression
function evaluate(expression) {
  const value = vm.runInContext(expression, context, { breakOnSigint: true });
  if (value !== undefined) {
    stdout.write(JSON.stringify(value));
  }
}

// Get runtime information
function info() {
  const info = {
    type: "SoftwareApplication",
    name: "Node.js",
    // Use `process.versions.node` rather than `process.version` which is prefixed with a v
    softwareVersion: process.versions.node,
    operatingSystem: `${os.type()} ${os.arch()} ${os.release()}`,
  };
  stdout.write(JSON.stringify(info));
}

// Get a list of packages available
async function packages() {
  const { execFileSync } = require("child_process");

  const out = execFileSync("npm", ["list", "--depth=0", "--json"]);
  const dependencies = JSON.parse(out).dependencies;

  for (const [name, { version }] of Object.entries(dependencies)) {
    const ssc = {
      type: "SoftwareSourceCode",
      programmingLanguage: "JavaScript",
      name,
      version,
    };

    stdout.write(`${JSON.stringify(ssc)}${END}\n`);
  }
}

// List variables in the context
function list() {
  for (const [name, value] of Object.entries(context)) {
    const [nativeType, nodeType, hint] = nodeTypesHint(value);

    const variable = {
      type: "Variable",
      name,
      programmingLanguage: "JavaScript",
      nativeType,
      nodeType,
      hint,
    };

    stdout.write(`${JSON.stringify(variable)}${END}\n`);
  }
}

// Get the types and hint for a value
function nodeTypesHint(value) {
  let nativeType;
  if (value === null) nativeType = "null";
  else if (Array.isArray(value)) nativeType = "array";
  else nativeType = typeof value;

  switch (nativeType) {
    case "undefined":
    case "null":
      return [nativeType, "Null", undefined];
    case "boolean":
      return [nativeType, "Boolean", value];
    case "number":
      return [nativeType, "Number", value];
    case "bigint":
      return [nativeType, "Integer", undefined]; // BigInt not serializable to JSON for hint
    case "string":
      return [
        nativeType,
        "String",
        { type: "StringHint", chars: [...value].length },
      ];
    case "array":
      return [nativeType, "Array", { type: "ArrayHint", length: value.length }];
    case "object":
      return [
        nativeType,
        ...(typeof value.type === "string"
          ? [value.type, undefined]
          : [
              "Object",
              {
                type: "ObjectHint",
                length: Object.keys(value).length,
                keys: Object.keys(value),
                values: Object.values(value).map(
                  (item) => nodeTypesHint(item)[2] ?? { type: "Unknown" }
                ),
              },
            ]),
      ];
    default:
      return [nativeType, "Object", { type: "Unknown" }];
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

// Fork the kernel instance
function fork(pipes) {
  const child = child_process.fork(__filename, [
    ...pipes,
    JSON.stringify(Object.values(imports)),
    JSON.stringify(context),
  ]);
  stdout.write(child.pid.toString());
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
          return execute(lines.slice(1));
        case EVAL:
          return evaluate(lines[1]);
        case INFO:
          return info();
        case PKGS:
          return packages();
        case LIST:
          return list();
        case GET:
          return get(lines[1]);
        case SET:
          return set(lines[1], lines[2]);
        case REMOVE:
          return remove(lines[1]);
        case FORK:
          return fork(lines.slice(1));
        default:
          throw new Error(`Unrecognized task ${lines[0]}`);
      }
    })();
  } catch (error) {
    if (error?.message === "Script execution was interrupted by `SIGINT`") {
      // Ignore error generated when interrupted
    } else {
      const msg = {
        type: "ExecutionMessage",
        level: "Exception",
        message: error.message ?? error.toString(),
        codeLocation: undefined,
        stackTrace: undefined,
      };

      if (error.name) msg.errorType = error.name;

      if (error.stack) {
        const stackLines = error.stack.split("\n");

        let stackTrace = "";
        for (const line of stackLines) {
          // Try to fine line and column of error in code
          const details = line.match(/evalmachine\.<anonymous>:(\d+):(\d+)/);
          if (details && !msg.codeLocation) {
            try {
              msg.codeLocation = {
                type: "CodeLocation",
                startLine: parseInt(details[1]) - 1,
                startColumn: parseInt(details[2]) - 1,
              };
            } catch {}
          }

          // Filter out lines related to evaluation
          if (
            !(
              line.includes("kernels/nodejs:") ||
              line.includes("node:vm:") ||
              line.includes("node:internal/readline/interface:") ||
              line.includes("node:events:")
            )
          ) {
            stackTrace +=
              line.replace("evalmachine.<anonymous>", "code") + "\n";
          }
        }
        msg.stackTrace = stackTrace;

        // If no code location found yet then try to get just line num
        if (!msg.codeLocation) {
          for (const line of stackLines) {
            const details = line.match(/evalmachine\.<anonymous>:(\d+)/);
            if (details) {
              try {
                msg.codeLocation = {
                  type: "CodeLocation",
                  startLine: parseInt(details[1]) - 1,
                };
                break;
              } catch {}
            }
          }
        }
      }

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
