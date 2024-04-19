/* eslint-disable import/no-unresolved */
/* eslint-disable @typescript-eslint/no-unused-vars */

import { Kernel } from "./kernel.js";
import { Plugin } from "./plugin.js";

class TestKernel extends Kernel {}

class TestPlugin extends Plugin {
  constructor() {
    super();

    this.kernels = {
      test: TestKernel,
    };
  }
}
