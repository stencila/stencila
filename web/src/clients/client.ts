/**
 * The base class for all clients
 *
 * TODO: This class should handle WebSocket disconnection
 * and reconnection.
 */
export abstract class Client {
  /**
   * The client's WebSocket connection
   */
  private ws: WebSocket;

  /**
   * Construct a new WebSocket client
   *
   * @param protocol The WebSocket protocol to use
   */
  constructor(protocol: string, params?: Record<string, string>) {
    const url = new URL(window.location.href);

    const doc = url.searchParams.get("doc");

    const query = params ? "?" + new URLSearchParams(params) : "";

    this.ws = new WebSocket(`ws://localhost:9000/~ws/${doc}${query}`, protocol);

    this.ws.onmessage = (event: MessageEvent<string>) => {
      const message = JSON.parse(event.data);

      if (process.env.NODE_ENV === "development") {
        console.log("🚩 Received message:", message);
      }

      this.receiveMessage(message);
    };
  }

  /**
   * Receive a message from the server
   *
   * This method should be overridden by clients that need to
   * handle incoming messages from the server.
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  receiveMessage(message: Record<string, unknown>) {}

  /**
   * Send a message to the server
   *
   * @param message The message as a JavaScript object
   */
  sendMessage(message: Record<string, unknown>) {
    if (process.env.NODE_ENV === "development") {
      console.log("📨 Sending message:", message);
    }

    this.ws.send(JSON.stringify(message));
  }
}
