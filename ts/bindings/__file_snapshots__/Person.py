class Person(Thing):
    """A person (alive, dead, undead, or fictional)."""

    address: Optional[str] = None
    """Postal address for the person."""

    affiliations: Optional[Array["Organization"]] = None
    """Organizations that the person is affiliated with."""

    emails: Optional[Array[str]] = None
    """Email addresses for the person."""

    familyNames: Optional[Array[str]] = None
    """Family name. In the U.S., the last name of a person."""

    funders: Optional[Array[Union["Organization", "Person"]]] = None
    """A person or organization that supports (sponsors) something through
some kind of financial contribution.
"""

    givenNames: Optional[Array[str]] = None
    """Given name. In the U.S., the first name of a person."""

    honorificPrefix: Optional[str] = None
    """An honorific prefix preceding a person's name such as Dr/Mrs/Mr."""

    honorificSuffix: Optional[str] = None
    """An honorific suffix after a person's name such as MD/PhD/MSCSW."""

    jobTitle: Optional[str] = None
    """The job title of the person (for example, Financial Manager)."""

    memberOf: Optional[Array["Organization"]] = None
    """An organization (or program membership) to which this person belongs."""

    telephoneNumbers: Optional[Array[str]] = None
    """Telephone numbers for the person."""


    def __init__(
        self,
        address: Optional[str] = None,
        affiliations: Optional[Array["Organization"]] = None,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[Union[str, Array["Node"]]] = None,
        emails: Optional[Array[str]] = None,
        familyNames: Optional[Array[str]] = None,
        funders: Optional[Array[Union["Organization", "Person"]]] = None,
        givenNames: Optional[Array[str]] = None,
        honorificPrefix: Optional[str] = None,
        honorificSuffix: Optional[str] = None,
        id: Optional[str] = None,
        identifiers: Optional[Array[Union[str, "PropertyValue"]]] = None,
        jobTitle: Optional[str] = None,
        memberOf: Optional[Array["Organization"]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        telephoneNumbers: Optional[Array[str]] = None,
        url: Optional[str] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            description=description,
            id=id,
            identifiers=identifiers,
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
