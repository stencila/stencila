#!/usr/bin/env deno

import { readLines } from 'https://deno.land/std@0.116.0/io/mod.ts'
import { encodeValue, encodeError } from './deno-codec.ts'

const READY = '\u{10ACDC}\n'
const RESULT = '\u{10CB40}\n'
const TASK = '\u{10ABBA}\n'

const textEncoder = new TextEncoder()

console.log = function (...args) {
  for (const arg of args) {
    const json = encodeValue(arg)
    Deno.stdout.write(textEncoder.encode(json + RESULT))
  }
}

Deno.stdout.write(textEncoder.encode(READY))
Deno.stderr.write(textEncoder.encode(READY))

for await (let task of readLines(Deno.stdin)) {
  const unescaped = task.replace(/\\n/g, '\n')

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
    Deno.stdout.write(textEncoder.encode(json + RESULT))
  }
  if (error !== null) {
    const json = encodeError(error.thrown)
    Deno.stderr.write(textEncoder.encode(json + RESULT))
  }
  Deno.stdout.write(textEncoder.encode(TASK))
  Deno.stderr.write(textEncoder.encode(TASK))
}
