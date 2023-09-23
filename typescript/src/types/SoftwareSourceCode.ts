// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from './CreativeWork';
import { SoftwareApplication } from './SoftwareApplication';
import { SoftwareSourceCodeOrSoftwareApplicationOrString } from './SoftwareSourceCodeOrSoftwareApplicationOrString';

// Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
export class SoftwareSourceCode extends CreativeWork {
  type = "SoftwareSourceCode";

  // Link to the repository where the un-compiled, human readable code and related
  // code is located.
  codeRepository?: string;

  // What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
  codeSampleType?: string;

  // The computer programming language.
  programmingLanguage?: string;

  // Runtime platform or script interpreter dependencies (Example - Java v1,
  // Python2.3, .Net Framework 3.0).
  runtimePlatform?: string[];

  // Dependency requirements for the software.
  softwareRequirements?: SoftwareSourceCodeOrSoftwareApplicationOrString[];

  // Target operating system or product to which the code applies.
  targetProducts?: SoftwareApplication[];

  constructor(options?: SoftwareSourceCode) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
