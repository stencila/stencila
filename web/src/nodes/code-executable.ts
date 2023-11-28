import { property } from "lit/decorators.js";

import { Executable } from "./executable";

export class CodeExecutable extends Executable {
  @property()
  code: string = "";

  @property()
  programmingLanguage?: string;
}
