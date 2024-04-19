/* eslint-disable import/no-unresolved */
/* eslint-disable @typescript-eslint/no-unused-vars */

import { Assistant } from "./assistant.js";
import { Plugin } from "./plugin.js";

class TestAssistant extends Assistant {}

class TestPlugin extends Plugin {
  constructor() {
    super();

    this.assistants = {
      test: new TestAssistant(),
    };
  }
}
