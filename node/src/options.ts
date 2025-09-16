/**
 * Shared type definitions for encode/decode options.
 *
 * This file exists to avoid duplication between the auto-generated bindings.d.ts
 * and the TypeScript source files that need to import these types. Both
 * convert.ts and bindings.ts import from here.
 */

/** Decoding options */
export interface DecodeOptions {
  /** The format to be decode from */
  format?: string
  /**
   * What to do if there are losses when decoding from the input
   *
   * Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or
   * a file path to write the losses to (`json` or `yaml` file extensions are supported).
   */
  losses?: string
}

/** Encoding options */
export interface EncodeOptions {
  /** The format to encode to */
  format?: string
  /**
   * Whether to encode as a standalone document
   *
   * Unless specified otherwise, this is the default when encoding to a file
   * (as opposed to a string).
   */
  standalone?: boolean
  /**
   * Whether to encode in compact form
   *
   * Some formats (e.g HTML and JSON) can be encoded in either compact
   * or "pretty-printed" (e.g. indented) forms.
   */
  compact?: boolean
  /**
   * What to do if there are losses when encoding to the output
   *
   * Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or
   * a file path to write the losses to (`json` or `yaml` file extensions are supported).
   */
  losses?: string
}
