/**
 * An operation on a string
 */
interface StringOp {
  /**
   * The position in the string from which the operation is applied
   */
  from: number;

  /**
   * The position in the string to which the operation is applied
   *
   * May be omitted for additions.
   */
  to?: number;

  /**
   * The string to insert between `from` and `to`.
   * For additions and replacements; may be omitted for deletions.
   */
  insert?: string;
}

/**
 * A patch to apply to a string
 */
interface StringPatch {
  /**
   * The version of the patch
   */
  version: number;

  /**
   * The operations in the patch
   */
  ops: StringOp[];
}

/**
 * A client that keeps a string synchronized with a buffer
 * on the server
 *
 * This client is read-only: it does not send patches to the
 * server, it only reads them.
 */
export class StringClient {
  /**
   * The client's websocket connection
   */
  private ws: WebSocket;

  /**
   * The local state of the string
   */
  private state: string = "";

  /**
   * The local version of the string
   *
   * Used to check for missed patches and request a
   * reset patch if necessary.
   */
  private version: number = 0;

  /**
   * A subscriber to the the string
   *
   * Is called whenever a patch is applied to the string `state`.
   */
  private subscriber?: (value: string) => void;

  /**
   * Construct a new `StringClient`
   *
   * @param format The format of the string (e.g. "html", "markdown")
   */
  constructor(format: string) {
    // TODO: Use a unique identifier for the document instance rather
    // than the pathname
    const { host, pathname } = window.location;
    const url = `ws://${host}${pathname}?format=${format}`;

    this.ws = new WebSocket(url, "sync-string.stencila.dev");

    this.ws.onmessage = (message) => {
      const { version, ops } = JSON.parse(message.data) as StringPatch;

      if (version != this.version + 1) {
        this.ws.send(JSON.stringify({ version: 0 }));
        return;
      }

      for (const op of ops) {
        const { from, to, insert } = op;

        if (to === undefined && insert !== undefined) {
          // Insert
          this.state =
            this.state.slice(0, from) + insert + this.state.slice(from);
        } else if (to !== undefined && insert === undefined) {
          // Delete
          this.state = this.state.slice(0, from) + this.state.slice(to);
        } else if (to !== undefined && insert !== undefined) {
          // Replace
          this.state =
            this.state.slice(0, from) + insert + this.state.slice(to);
        } else if (to === 0 && from == 0 && insert !== undefined) {
          // Reset
          this.state = insert;
        }
      }

      this.version = version;

      if (this.subscriber) {
        this.subscriber(this.state);
      }
    };
  }

  /**
   * Subscribe to changes in the string
   *
   * @param subscriber The subscriber function which will be called
   *                   with the string, each time it changes
   */
  subscribe(subscriber: (value: string) => void) {
    this.subscriber = subscriber;
  }
}
