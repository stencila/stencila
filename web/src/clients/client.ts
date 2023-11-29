import { type DocumentId } from "../types";

/**
 * The abstract base class for all clients
 */
export abstract class Client {
  /**
   * The client's WebSocket connection
   */
  private ws: WebSocket;

  /**
   * Construct a new document client
   *
   * @param id  The id of the document
   * @param subprotocol The WebSocket subprotocol to use
   */
  constructor(id: DocumentId, subprotocol: string) {
    const protocol = window.location.protocol === "http:" ? "ws" : "wss";
    const host = window.location.host
    this.ws = new WebSocket(
      `${protocol}://${host}/~ws/${id}`,
      subprotocol + ".stencila.org"
    );

    this.ws.onmessage = (event: MessageEvent<string>) => {
      const message = JSON.parse(event.data);

      if (process.env.NODE_ENV === "development") {
        console.log("🚩 Received:", message);
      }

      this.receiveMessage(message);
    };
  }

  /**
   * Receive a message from the server
   *
   * This method should be overridden by clients that need to
   * handle incoming messages from the server.
   * 
   * @param message The message as a JavaScript object
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  protected receiveMessage(message: Record<string, unknown>) { }

  /**
   * Send a message to the server
   *
   * @param message The message as a JavaScript object
   */
  protected sendMessage(message: Record<string, unknown>) {
    if (process.env.NODE_ENV === "development") {
      console.log("📨 Sending:", message);
    }

    this.ws.send(JSON.stringify(message));
  }
}
