// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Organization } from "./Organization.js";
import { PersonOrOrganization } from "./PersonOrOrganization.js";
import { PostalAddressOrString } from "./PostalAddressOrString.js";
import { Thing } from "./Thing.js";

/**
 * A person (alive, dead, undead, or fictional).
 */
export class Person extends Thing {
  type = "Person";

  /**
   * Postal address for the person.
   */
  address?: PostalAddressOrString;

  /**
   * Organizations that the person is affiliated with.
   */
  affiliations?: Organization[];

  /**
   * Email addresses for the person.
   */
  emails?: string[];

  /**
   * Family name. In the U.S., the last name of a person.
   */
  familyNames?: string[];

  /**
   * A person or organization that supports (sponsors) something through some kind of financial contribution.
   */
  funders?: PersonOrOrganization[];

  /**
   * Given name. In the U.S., the first name of a person.
   */
  givenNames?: string[];

  /**
   * An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
   */
  honorificPrefix?: string;

  /**
   * An honorific suffix after a person's name such as MD/PhD/MSCSW.
   */
  honorificSuffix?: string;

  /**
   * The job title of the person (for example, Financial Manager).
   */
  jobTitle?: string;

  /**
   * An organization (or program membership) to which this person belongs.
   */
  memberOf?: Organization[];

  /**
   * Telephone numbers for the person.
   */
  telephoneNumbers?: string[];

  constructor(options?: Partial<Person>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Person`
*/
export function person(options?: Partial<Person>): Person {
  return new Person(options);
}
