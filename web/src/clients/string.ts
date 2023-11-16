import { Client } from "./client";

/**
 * An operation on a string
 */
export interface StringOp {
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
export interface StringPatch {
  /**
   * The version of the patch
   */
  version: number;

  /**
   * The operations in the patch
   */
  ops?: StringOp[];
}

/**
 * A client that keeps a string synchronized with a buffer
 * on the server
 */
export class StringClient extends Client {
  /**
   * The local state of the string
   */
  protected state: string = "";

  /**
   * The local version of the string
   *
   * Used to check for missed patches and request a
   * reset patch if necessary.
   */
  protected version: number = 0;

  /**
   * A subscriber to the the string
   *
   * A function that is called whenever a patch is applied to the
   * string `state`.
   */
  protected subscriber?: (value: string) => void;

  /**
   * Construct a new `StringClient`
   *
   * @param format The format of the string (e.g. "html", "markdown")
   */
  constructor(format: string) {
    super("sync-string.stencila.dev", {format});
  }

  /**
   * Receive a message from the server
   *
   * @override
   */
  receiveMessage(message: Record<string, unknown>) {
    const { version, ops } = message as unknown as StringPatch;

    // Is the patch a reset patch?
    const isReset = ops.length === 1 && ops[0].from === 0 && ops[0].to === 0;

    // Check for non-sequential patch and request a reset patch if necessary
    if (!isReset && version != this.version + 1) {
      this.sendMessage({ version: 0 });
      return;
    }

    // Apply each operation in the patch
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
        this.state = this.state.slice(0, from) + insert + this.state.slice(to);
      } else if (to === 0 && from == 0 && insert !== undefined) {
        // Reset
        this.state = insert;
      }
    }

    // Update local version number
    this.version = version;

    // Notify the subscriber (if any)
    if (this.subscriber) {
      this.subscriber(this.state);
    }
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
