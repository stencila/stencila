/* eslint-disable import/no-unresolved */
/* eslint-disable @typescript-eslint/no-unused-vars */

import http from "http";
import readline from "readline";

import {
  ExecutionMessage,
  Node,
  SoftwareApplication,
  SoftwareSourceCode,
  Variable,
} from "@stencila/types";

import {
  Assistant,
  AssistantName,
  GenerateOptions,
  GenerateOutput,
  GenerateTask,
} from "./assistant.js";
import { Codec, CodecName, DecodeInfo, EncodeInfo } from "./codec.js";
import { KernelInstanceName, KernelName, Kernel } from "./kernel.js";

/**
 * A base plugin class for Stencila plugins built with Node.js
 *
 * Plugin developers can extend this class and override the `protected`
 * methods for their use case.
 */
export class Plugin {
  /**
   * The codecs the plugin provides
   */
  codecs: Record<CodecName, Codec> = {};

  /**
   * The kernels the plugin provides
   */
  kernels: Record<KernelName, new () => Kernel> = {};

  /**
   * The instances of kernels create by the plugin
   */
  kernelInstances: Record<KernelInstanceName, Kernel> = {};

  /**
   * The assistants the plugin provides
   */
  assistants: Record<AssistantName, Assistant> = {};

  /**
   * Get the health of the plugin
   *
   * At present this method is only used to check communication with
   * the plugin. In the future, the expected response object may be used
   * for more detailed statistics about resource usage etc by the plugin.
   */
  protected async health() {
    return {
      timestamp: Math.floor(Date.now() / 1000),
      status: "OK",
    };
  }

  /**
   * JSON-RPC interface for `Codec.fromString`
   *
   * @param {Object}
   * @param content The content to decode
   * @param codec The name of the codec to use
   *
   * @return GenerateOutput
   */
  protected async codec_from_string({
    content,
    codec,
  }: {
    content: string;
    codec: CodecName;
  }): Promise<[Node, DecodeInfo]> {
    const instance = this.codecs[codec];
    if (instance === undefined) {
      throw new Error(`No codec named '${codec}'`);
    }

    return instance.fromString(content);
  }

  /**
   * JSON-RPC interface for `Codec.toString`
   *
   * @param {Object}
   * @param node The node to encode to a string
   * @param codec The name of the codec to use
   *
   * @return GenerateOutput
   */
  protected async codec_to_string({
    node,
    codec,
  }: {
    node: Node;
    codec: CodecName;
  }): Promise<[string, EncodeInfo]> {
    const instance = this.codecs[codec];
    if (instance === undefined) {
      throw new Error(`No codec named '${codec}'`);
    }

    return instance.toString(node);
  }

  /**
   * JSON-RPC interface for `Kernel.start`
   *
   * @param {Object}
   * @property kernel The name of the kernel to start an instance for
   *
   * @returns {Object}
   * @property instance The name of the kernel instance that was started
   */
  protected async kernel_start({ kernel }: { kernel: KernelName }): Promise<{
    instance: KernelInstanceName;
  }> {
    const Kernel = this.kernels[kernel];
    if (Kernel === undefined) {
      throw new Error(`No kernel named '${kernel}'`);
    }

    const instance = new Kernel();
    instance.start();

    const instanceName = `${kernel}-${this.kernels.length}`;
    this.kernelInstances[instanceName] = instance;

    return { instance: instanceName };
  }

  /**
   * JSON-RPC interface for `Kernel.stop`
   *
   * @param {Object}
   * @property instance The name of the kernel instance to stop
   */
  protected async kernel_stop({
    instance,
  }: {
    instance: KernelInstanceName;
  }): Promise<void> {
    return await this.kernelInstances[instance].stop();
  }

  /**
   * JSON-RPC interface for `Kernel.info`
   *
   * @param {Object}
   * @property instance The name of the kernel instance to get information for
   */
  protected async kernel_info({
    instance,
  }: {
    instance: KernelInstanceName;
  }): Promise<SoftwareApplication> {
    return await this.kernelInstances[instance].info();
  }

  /**
   * JSON-RPC interface for `Kernel.packages`
   *
   * @param {Object}
   * @property instance The name of the kernel instance to list packages for
   */
  protected async kernel_packages({
    instance,
  }: {
    instance: KernelInstanceName;
  }): Promise<SoftwareSourceCode[]> {
    return await this.kernelInstances[instance].packages();
  }

  /**
   * JSON-RPC interface for `Kernel.execute`
   *
   * @param {Object}
   * @property code The code to execute
   * @property instance The name of the kernel instance to execute the code in
   *
   * @return {Object}
   * @property outputs The outputs from executing the code
   * @property messages The messages associated with executing the code
   */
  protected async kernel_execute({
    code,
    instance,
  }: {
    code: string;
    instance: KernelInstanceName;
  }): Promise<{
    outputs: Node[];
    messages: ExecutionMessage[];
  }> {
    return await this.kernelInstances[instance].execute(code);
  }

  /**
   * JSON-RPC interface for `Kernel.evaluate`
   *
   * @param {Object}
   * @property code The code to evaluate
   * @property instance The name of the kernel instance to evaluate the code in
   *
   * @return {Object}
   * @property output The output from evaluating the code
   * @property messages The messages associated with evaluating the code
   */
  protected async kernel_evaluate({
    code,
    instance,
  }: {
    code: string;
    instance: KernelInstanceName;
  }): Promise<{
    output: Node;
    messages: ExecutionMessage[];
  }> {
    return await this.kernelInstances[instance].evaluate(code);
  }

  /**
   * JSON-RPC interface for `Kernel.list`
   *
   * @param {Object}
   * @property instance The name of the kernel instance to list variables for
   */
  protected async kernel_list({
    instance,
  }: {
    instance: KernelInstanceName;
  }): Promise<Variable[]> {
    return await this.kernelInstances[instance].list();
  }

  /**
   * JSON-RPC interface for `Kernel.get`
   *
   * @param {Object}
   * @property name The name of the variable
   * @property instance The name of the kernel instance get the variable from
   */
  protected async kernel_get({
    name,
    instance,
  }: {
    name: string;
    instance: KernelInstanceName;
  }): Promise<Variable | null> {
    return await this.kernelInstances[instance].get(name);
  }

  /**
   * JSON-RPC interface for `Kernel.set`
   *
   * @param {Object}
   * @property name The name of the variable
   * @property value The value of the node
   * @property instance The name of the kernel instance to list variables for
   */
  protected async kernel_set({
    name,
    value,
    instance,
  }: {
    name: string;
    value: Node;
    instance: KernelInstanceName;
  }): Promise<void> {
    return await this.kernelInstances[instance].set(name, value);
  }

  /**
   * JSON-RPC interface for `Kernel.remove`
   *
   * @param {Object}
   * @property name The name of the variable
   * @property instance The name of the kernel instance to remove the variable from
   */
  protected async kernel_remove({
    name,
    instance,
  }: {
    name: string;
    instance: KernelInstanceName;
  }): Promise<void> {
    return await this.kernelInstances[instance].remove(name);
  }

  /**
   * JSON-RPC interface for `Assistant.systemPrompt`
   *
   * @param {Object}
   * @param task The task to create a system prompt template for
   * @param options Options for generation
   * @param assistant The name of the assistant that should create the system prompt
   *
   * @return string
   */
  protected async assistant_system_prompt({
    task,
    options,
    assistant,
  }: {
    task: GenerateTask;
    options: GenerateOptions;
    assistant: AssistantName;
  }): Promise<string> {
    const instance = this.assistants[assistant];
    if (instance === undefined) {
      throw new Error(`No assistant named '${assistant}'`);
    }

    return await instance.systemPrompt(task, options);
  }

  /**
   * JSON-RPC interface for `Assistant.performTask`
   *
   * @param {Object}
   * @param task The task to perform
   * @param options Options for generation
   * @param assistant The name of the assistant that should perform the task
   *
   * @return GenerateOutput
   */
  protected async assistant_perform_task({
    task,
    options,
    assistant,
  }: {
    task: GenerateTask;
    options: GenerateOptions;
    assistant: AssistantName;
  }): Promise<GenerateOutput> {
    const instance = this.assistants[assistant];
    if (instance === undefined) {
      throw new Error(`No assistant named '${assistant}'`);
    }

    return await instance.performTask(task, options);
  }

  /**
   * Handle a JSON-RPC request and return a JSON-RPC response
   */
  private async handleRequest(requestJson: string): Promise<string> {
    let request;
    try {
      request = JSON.parse(requestJson);
    } catch (error) {
      // Generate parsing error
      return errorResponse(null, -32700, "Parse error");
    }

    const { id, method, params } = request;

    // Check if the method exists and is callable
    // @ts-expect-error because indexing this by string
    const func = this[method];
    if (typeof func === "function") {
      try {
        const result = await func.call(this, params);
        return successResponse(id, result);
      } catch (error) {
        return errorResponse(id, -32603, `Internal error: ${error}`);
      }
    } else {
      return errorResponse(id, -32601, `Method \`${method}\` not found`);
    }

    function successResponse(id: string, result: unknown): string {
      // Result must always be defined (i.e. not `undefined`) for success responses
      return JSON.stringify({ jsonrpc: "2.0", id, result: result ?? null });
    }

    function errorResponse(
      id: string | null,
      code: number,
      message: string,
    ): string {
      return JSON.stringify({ jsonrpc: "2.0", id, error: { code, message } });
    }
  }

  /**
   * Listen for JSON-RPC requests on standard input and send responses on standard output
   */
  private async listenStdio(): Promise<void> {
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false,
    });

    rl.on("line", async (requestJson) => {
      const responseJson = await this.handleRequest(requestJson);
      console.log(responseJson);
    });
  }

  /**
   * Listen for JSON-RPC requests on HTTP
   */
  private async listenHttp(port: number, token: string): Promise<void> {
    const server = http.createServer(async (req, res) => {
      // Check the request is from localhost
      if (
        req.socket.remoteAddress === "127.0.0.1" ||
        req.socket.remoteAddress === "::1"
      ) {
        res.writeHead(403);
        res.end("Access denied");
        return;
      }

      // Check for the bearer token in the Authorization header
      const authHeader = req.headers["authorization"];
      const receivedToken =
        authHeader && authHeader.split(" ")[0] === "Bearer"
          ? authHeader.split(" ")[1]
          : null;
      if (!receivedToken || receivedToken !== token) {
        res.writeHead(401);
        res.end("Invalid or missing token");
        return;
      }

      if (
        req.method === "POST" &&
        req.headers["content-type"] === "application/json"
      ) {
        // Handle the request
        let body = "";
        req.on("data", (chunk) => {
          body += chunk.toString();
        });
        req.on("end", async () => {
          try {
            const responseJson = await this.handleRequest(body);
            res.setHeader("Content-Type", "application/json");
            res.writeHead(200);
            res.end(responseJson);
          } catch (error) {
            // Handle any errors not handled in `handleRequest`
            res.writeHead(500);
            res.end(
              JSON.stringify({
                jsonrpc: "2.0",
                error: { code: -32603, message: "Internal error" },
                id: null,
              }),
            );
          }
        });
      } else {
        // Respond with 405 Method Not Allowed if not a POST request and JSON payload
        res.writeHead(405);
        res.end();
      }
    });

    server.listen(port);
  }

  /**
   * Run the plugin based on environment variables
   */
  public async run(): Promise<void> {
    const protocol = process.env.STENCILA_TRANSPORT ?? "stdio";
    if (protocol == "stdio") {
      this.listenStdio();
    } else if (protocol == "http") {
      this.listenHttp(
        parseInt(process.env.STENCILA_PORT!),
        process.env.STENCILA_TOKEN!,
      );
    } else {
      throw Error(`Unknown protocol: ${protocol}`);
    }
  }
}
