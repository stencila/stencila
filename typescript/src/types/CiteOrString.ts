// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Cite } from './Cite'

// `Cite` or `string`
export type CiteOrString =
  Cite |
  string;

export function citeOrString(other: CiteOrString): CiteOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as CiteOrString;
  }
  switch(other.type) {
    case "Cite": return Cite.from(other as Cite);
    default: throw new Error(`Unexpected type for CiteOrString: ${other.type}`)
  }
}
