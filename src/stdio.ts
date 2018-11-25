#!/usr/bin/env node

import * as readline from 'readline'

import handle from './handle'

/**
 * A JSON-RPC server using standard input/output
 * for communication.
 */
export const stdio = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  prompt: ''
})
.on('line', request => console.log(handle(request)))
