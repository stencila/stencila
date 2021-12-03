#!/usr/bin/env deno

import { readLines } from 'https://deno.land/std@0.116.0/io/mod.ts'
import { encodeValue, encodeError } from './deno-codec.ts'

const resSep = '\u{10ABBA}\n'
const transSep = '\u{10ACDC}\n'
const textEncoder = new TextEncoder()

console.log = function (...args) {
  for (const arg of args) {
    const json = encodeValue(arg)
    Deno.stdout.write(textEncoder.encode(json + resSep))
  }
}

for await (let code of readLines(Deno.stdin)) {
  const unescaped = code.replace(/\\n/g, '\n')

  const { files } = await Deno.emit('/code.ts', {
    // Do not check the Typescript, just strip it of type annotations etc
    check: false,
    sources: { '/code.ts': unescaped },
  })
  let transpiled = files['file:///code.ts.js']
  if (transpiled == undefined) {
    // Syntax error, so pass on to get error
    transpiled = unescaped
  }

  // @ts-expect-error because `evalContext` is not part of public API
  const [value, error] = Deno.core.evalContext(transpiled)
  if (value !== null && value !== undefined) {
    const json = encodeValue(value)
    Deno.stdout.write(textEncoder.encode(json + resSep))
  }
  if (error !== null) {
    const json = encodeError(error.thrown)
    Deno.stderr.write(textEncoder.encode(json + resSep))
  }
  Deno.stdout.write(textEncoder.encode(transSep))
  Deno.stderr.write(textEncoder.encode(transSep))
}
