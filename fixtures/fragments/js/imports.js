// Example imports from https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import

import defaultExport from "module-1";
import * as name from "module-2";
import { export1 } from "module-3";
import { export1 as alias } from "module-4";
import { export1 , export2 } from "module-5";
import { export1 , export2 as alias } from "module-6";
import defaultExport, { export1 } from "module-7";
import defaultExport, * as name from "module-8";
import "module-9";
var promise = import("module-10");

// Node require calls
let defaultExport = require("module-11")
let alias = require("module-12").export1;

// Imports that are not detected

const foo = import("foo" + "bar")

const pkg_var = "foo"
const bar = require(pkg_var)
