import { StringClient } from "./string";
import morphdom from "morphdom";

/**
 * A client that keeps a DOM element synchronized with a
 * HTML buffer on the server
 */
export class DomClient extends StringClient {
  /**
   * Construct a new `DomClient`
   * 
   * @param elem An `HTMLElement` or a CSS selector
   */
  constructor(elem: string | HTMLElement) {
    super("html");

    let target: HTMLElement;
    if (typeof elem === "string") {
      target = document.querySelector(elem);
    } else {
      target = elem;
    }

    this.subscribe((html) => morphdom(target, html));
  }
}
