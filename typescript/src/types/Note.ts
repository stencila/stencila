// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Block } from './Block';
import { NoteType } from './NoteType';
import { String } from './String';

// Additional content which is not part of the main content of a document.
export class Note {
  // The type of this item
  type = "Note";

  // The identifier for this item
  id?: String;

  // Determines where the note content is displayed within the document.
  noteType: NoteType;

  // Content of the note, usually a paragraph.
  content: Block[];

  constructor(noteType: NoteType, content: Block[], options?: Note) {
    if (options) Object.assign(this, options)
    this.noteType = noteType;
    this.content = content;
  }
}
