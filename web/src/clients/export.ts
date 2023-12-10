import { DocumentId } from "../types";

/**
 * A client for exporting a document as a specific format
 */
export class ExportClient {
  /**
   * The URL that will be fetched from
   */
  url: string;

  constructor(
    doc: DocumentId,
    format = "json",
    options: {
      dom?: boolean;
    } = {},
  ) {
    this.url = `/~export/${doc}?format=${format}&dom=${options.dom ?? false}`;
  }

  /**
   * Fetch the exported content as text
   */
  async fetch() {
    const response = await fetch(this.url);
    const content = await response.text();
    return content;
  }
}
