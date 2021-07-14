class Person(Thing):
    """A person (alive, dead, undead, or fictional)."""

    address: Optional[Union["PostalAddress", String]] = None
    """Postal address for the person."""

    affiliations: Optional[Array["Organization"]] = None
    """Organizations that the person is affiliated with."""

    emails: Optional[Array[String]] = None
    """Email addresses for the person."""

    familyNames: Optional[Array[String]] = None
    """Family name. In the U.S., the last name of a person."""

    funders: Optional[Array[Union["Organization", "Person"]]] = None
    """A person or organization that supports (sponsors) something through
some kind of financial contribution.
"""

    givenNames: Optional[Array[String]] = None
    """Given name. In the U.S., the first name of a person."""

    honorificPrefix: Optional[String] = None
    """An honorific prefix preceding a person's name such as Dr/Mrs/Mr."""

    honorificSuffix: Optional[String] = None
    """An honorific suffix after a person's name such as MD/PhD/MSCSW."""

    jobTitle: Optional[String] = None
    """The job title of the person (for example, Financial Manager)."""

    memberOf: Optional[Array["Organization"]] = None
    """An organization (or program membership) to which this person belongs."""

    telephoneNumbers: Optional[Array[String]] = None
    """Telephone numbers for the person."""


    def __init__(
        self,
        address: Optional[Union["PostalAddress", String]] = None,
        affiliations: Optional[Array["Organization"]] = None,
        alternateNames: Optional[Array[String]] = None,
        description: Optional[Union[Array["BlockContent"], Array["InlineContent"], String]] = None,
        emails: Optional[Array[String]] = None,
        familyNames: Optional[Array[String]] = None,
        funders: Optional[Array[Union["Organization", "Person"]]] = None,
        givenNames: Optional[Array[String]] = None,
        honorificPrefix: Optional[String] = None,
        honorificSuffix: Optional[String] = None,
        id: Optional[String] = None,
        identifiers: Optional[Array[Union["PropertyValue", String]]] = None,
        images: Optional[Array[Union["ImageObject", String]]] = None,
        jobTitle: Optional[String] = None,
        memberOf: Optional[Array["Organization"]] = None,
        meta: Optional[Object] = None,
        name: Optional[String] = None,
        telephoneNumbers: Optional[Array[String]] = None,
        url: Optional[String] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            description=description,
            id=id,
            identifiers=identifiers,
            images=images,
            meta=meta,
            name=name,
            url=url
        )
        if address is not None:
            self.address = address
        if affiliations is not None:
            self.affiliations = affiliations
        if emails is not None:
            self.emails = emails
        if familyNames is not None:
            self.familyNames = familyNames
        if funders is not None:
            self.funders = funders
        if givenNames is not None:
            self.givenNames = givenNames
        if honorificPrefix is not None:
            self.honorificPrefix = honorificPrefix
        if honorificSuffix is not None:
            self.honorificSuffix = honorificSuffix
        if jobTitle is not None:
            self.jobTitle = jobTitle
        if memberOf is not None:
            self.memberOf = memberOf
        if telephoneNumbers is not None:
            self.telephoneNumbers = telephoneNumbers
