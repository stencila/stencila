// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from "./CreativeWork.js";
import { SoftwareApplication } from "./SoftwareApplication.js";
import { SoftwareSourceCodeOrSoftwareApplicationOrString } from "./SoftwareSourceCodeOrSoftwareApplicationOrString.js";

/**
 * Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
 */
export class SoftwareSourceCode extends CreativeWork {
  type = "SoftwareSourceCode";

  /**
   * The name of the item.
   */
  name: string;

  /**
   * The computer programming language.
   */
  programmingLanguage: string;

  /**
   * Link to the repository where the un-compiled, human readable code and related code is located.
   */
  codeRepository?: string;

  /**
   * What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
   */
  codeSampleType?: string;

  /**
   * Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0).
   */
  runtimePlatform?: string[];

  /**
   * Dependency requirements for the software.
   */
  softwareRequirements?: SoftwareSourceCodeOrSoftwareApplicationOrString[];

  /**
   * Target operating system or product to which the code applies.
   */
  targetProducts?: SoftwareApplication[];

  constructor(name: string, programmingLanguage: string, options?: Partial<SoftwareSourceCode>) {
    super();
    if (options) Object.assign(this, options);
    this.name = name;
    this.programmingLanguage = programmingLanguage;
  }
}

/**
* Create a new `SoftwareSourceCode`
*/
export function softwareSourceCode(name: string, programmingLanguage: string, options?: Partial<SoftwareSourceCode>): SoftwareSourceCode {
  return new SoftwareSourceCode(name, programmingLanguage, options);
}
