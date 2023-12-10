import { DocumentId } from "../types";

/**
 * A client for exporting a document as a specific format
 */
export class ExportClient {
  /**
   * The URL that will be fetched from
   */
  url: string;

  constructor(doc: DocumentId, format: string = "json") {
    this.url = `/~export/${doc}?format=${format}`;
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
