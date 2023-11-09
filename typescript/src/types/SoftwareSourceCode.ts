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
   * Link to the repository where the un-compiled, human readable code and related code is located.
   */
  codeRepository?: string;

  /**
   * What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
   */
  codeSampleType?: string;

  /**
   * The computer programming language.
   */
  programmingLanguage?: string;

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

  constructor(options?: Partial<SoftwareSourceCode>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `SoftwareSourceCode`
*/
export function softwareSourceCode(options?: Partial<SoftwareSourceCode>): SoftwareSourceCode {
  return new SoftwareSourceCode(options);
}
