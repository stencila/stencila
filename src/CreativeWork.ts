import { Date, Number, Text, URL } from './dataTypes'
import { type, property } from './decorators'
import Organization from './Organization'
import Person from './Person'
import Thing from './Thing'

/**
 * The most generic kind of creative work, including books, movies,
 * photographs, software programs, etc.
 * 
 * @see {@link https://schema.org/CreativeWork}
 */
@type('schema:CreativeWork')
export default class CreativeWork extends Thing {

  /**
   * The author of this content or rating. Please note that author is special in
   * that HTML 5 provides a special mechanism for indicating authorship via the rel
   * tag. That is equivalent to this and may be used interchangeably.
   * 
   * @see {@link https://schema.org/author}
   */
  @property('schema:author')
  authors: Array<Organization | Person> = []

  /**
   * A secondary contributor to the CreativeWork or Event.
   * 
   * @see {@link https://schema.org/contributor}
   */
  @property('schema:contributor')
  contributors: Array<Organization | Person> = []

  /**
   * The creator/author of this CreativeWork. This is the same as
   * the Author property for CreativeWork.
   * 
   * @see {@link https://schema.org/creator}
   */
  @property('schema:creator')
  creators: Array<Organization | Person> = []

  /**
   * Date of first broadcast/publication.
   * 
   * @see {@link https://schema.org/datePublished}
   */
  @property('schema:datePublished')
  datePublished: Date = ''

  /**
   * Keywords or tags used to describe this content.
   * Multiple entries in a keywords list are typically delimited by commas.
   * 
   * @see {@link https://schema.org/keywords}
   */
  @property('schema:keywords')
  keywords: Text = ''

  /**
   * A license document that applies to this content, typically indicated by URL.
   * 
   * @see {@link https://schema.org/license}
   */
  @property('schema:license')
  license: CreativeWork | URL = ''

  /**
   * The textual content of this CreativeWork.
   * 
   * @see {@link https://schema.org/text}
   */
  @property('schema:text')
  text: Text = ''

  /**
   * The version of the CreativeWork embodied by a specified resource.
   * 
   * @see {@link https://schema.org/version}
   */
  @property('schema:version')
  version: Number | Text = ''

}
