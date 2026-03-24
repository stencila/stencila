// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * An icon, typically rendered using an icon font.
 */
export class Icon extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Icon";

  /**
   * The name of the icon e.g. "clock" or "lucide:clock".
   */
  name: string;

  /**
   * An accessible text label for the icon.
   */
  label?: string;

  /**
   * Whether the icon is purely decorative.
   */
  decorative?: boolean;

  /**
   * Tailwind utility classes to apply to the icon.
   */
  style?: string;

  constructor(name: string, options?: Partial<Icon>) {
    super();
    this.type = "Icon";
    if (options) Object.assign(this, options);
    this.name = name;
  }
}

/**
* Create a new `Icon`
*/
export function icon(name: string, options?: Partial<Icon>): Icon {
  return new Icon(name, options);
}
