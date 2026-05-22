// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Thing } from "./Thing.js";

/**
 * A container image used by a computational action.
 */
export class ContainerImage extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ContainerImage";

  /**
   * The name of the item.
   */
  name: string;

  /**
   * The specific image type, such as DockerImage or SIFImage.
   */
  additionalType?: string;

  /**
   * The registry or service that hosts the image.
   */
  registry?: string;

  /**
   * The image tag.
   */
  tag?: string;

  /**
   * The SHA-256 checksum of the image.
   */
  sha256?: string;

  constructor(name: string, options?: Partial<ContainerImage>) {
    super();
    this.type = "ContainerImage";
    if (options) Object.assign(this, options);
    this.name = name;
  }
}

/**
* Create a new `ContainerImage`
*/
export function containerImage(name: string, options?: Partial<ContainerImage>): ContainerImage {
  return new ContainerImage(name, options);
}
