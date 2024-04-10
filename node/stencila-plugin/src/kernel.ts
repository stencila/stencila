/* eslint-disable @typescript-eslint/no-unused-vars */

import {
  ExecutionMessage,
  Node,
  SoftwareApplication,
  SoftwareSourceCode,
  Variable,
} from "@stencila/types";

/**
 * The type for the name of a kernel
 */
export type KernelName = string;

/**
 * The type for the name of a kernel instance
 */
export type KernelInstanceName = string;

export abstract class Kernel {
  /**
   * Start the kernel instance
   *
   * This method is called by Stencila when a kernel instance is
   * started. This method only needs to be implemented for plugins that provide
   * need to instantiate some state for a kernel instance (i.e. a kernel that is capable
   * of storing variables).
   */
  async start(): Promise<void> {}

  /**
   * Stop the kernel instance
   *
   * This method is called by Stencila when a kernel instance is
   * stopped because it is no longer needed. Because Stencila will also
   * stop the plugin instance at that time, this method only needs to be
   * implemented for plugins that host more than one kernel instance at a time,
   * or that need to perform clean up for a stopped kernel instance.
   *
   * This default implementation does nothing.
   */
  async stop(): Promise<void> {}

  /**
   * Get information about the kernel instance
   *
   * This method is called by Stencila to obtain information about a
   * kernel instance while it is running. It must be implemented by
   * all plugins that provide one or more kernels.
   *
   * This default implementation throws an error to indicate that
   * it has not been overridden.
   */
  async info(): Promise<SoftwareApplication> {
    throw Error(
      "Method `kernelInfo` must be overridden by plugins that provide one or more kernels",
    );
  }

  /**
   * Get a list of packages available in the kernel instance
   *
   * This method is called by Stencila to obtain a list of packages
   * available in a kernel instance. This is used for improving
   * assistant code generation (reducing hallucination of packages)
   * and other purposes. This method should be implemented by plugins
   * that provide kernels which have the concept of installable packages.
   *
   * This default implementation returns an empty list.
   */
  async packages(): Promise<SoftwareSourceCode[]> {
    return [];
  }

  /**
   * Execute code in the kernel instance
   *
   * This method is called by Stencila when executing `CodeChunk`s.
   * It should be implemented for most kernels. If the plugin provides
   * more than one kernel, this method will need to branch based on the
   * type of the kernel instance.
   *
   * This default implementation returns no outputs or messages.
   *
   * @param code The code to execute
   *
   * @return {Object}
   * @property outputs The outputs from executing the code
   * @property messages The messages associated with executing the code
   */
  async execute(code: string): Promise<{
    outputs: Node[];
    messages: ExecutionMessage[];
  }> {
    return {
      outputs: [],
      messages: [],
    };
  }

  /**
   * Evaluate code in the kernel instance
   *
   * This method is called by Stencila when evaluating code expressions
   * in `CodeExpression`, `ForBlock` and other node types.
   * It should be implemented for most kernels. If the plugin provides
   * more than one kernel, this method will need to branch based on the
   * type of the kernel instance.
   *
   * This default implementation returns no output or messages.
   *
   * @param code The code to evaluate
   *
   * @return {Object}
   * @property output The output from evaluating the code
   * @property messages The messages associated with evaluating the code
   */
  async evaluate(code: string): Promise<{
    output: Node;
    messages: ExecutionMessage[];
  }> {
    return {
      output: [],
      messages: [],
    };
  }

  /**
   * Get a list of variables available in the kernel instance
   *
   * This method is called by Stencila to obtain a list of variables
   * available in a kernel instance. This is used for improving
   * assistant code generation (reducing hallucination of variables)
   * and other purposes. This method should be implemented by plugins
   * that provide kernels which maintain variables as part of the kernel
   * state.
   *
   * This default implementation returns an empty list.
   */
  async list(): Promise<Variable[]> {
    return [];
  }

  /**
   * Get a variable from the kernel instance
   *
   * This method is called by Stencila to obtain a variables so it
   * can be displayed or "mirrored" to another kernel. This method should
   * be implemented by plugins that provide kernels which maintain variables
   * as part of the kernel state.
   *
   * This default implementation returns `null` (the return value when a
   * variable does not exist).
   *
   * @param name The name of the variable
   */
  async get(name: string): Promise<Variable | null> {
    return null;
  }

  /**
   * Set a variable in the kernel instance
   *
   * This method is called by Stencila to set `Parameter` values or
   * to "mirror" variable from another kernel. This method should
   * be implemented by plugins that provide kernels which maintain variables
   * as part of the kernel state.
   *
   * This default implementation does nothing.
   *
   * @param name The name of the variable
   * @param value The value of the node
   */
  async set(name: string, value: Node): Promise<void> {}

  /**
   * Remove a variable from the kernel instance
   *
   * This method is called by Stencila to keep the variables in a kernel
   * instance in sync with the variables defined in the code in a document.
   * For example, if a `CodeChunk` that declares a variable is removed from
   * from the document, then the variable should be removed from the kernel
   * (so that it is not accidentally reused later).
   *
   * This default implementation does nothing.
   *
   * @param name The name of the variable
   */
  async remove(name: string): Promise<void> {}
}
