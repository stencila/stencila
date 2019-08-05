class Person(Thing):
    """A person (alive, dead, undead, or fictional)."""

    address: Optional[str]
    affiliations: Optional[Array["Organization"]]
    emails: Optional[Array[str]]
    familyNames: Optional[Array[str]]
    funders: Optional[Array[Union["Organization", "Person"]]]
    givenNames: Optional[Array[str]]
    honorificPrefix: Optional[str]
    honorificSuffix: Optional[str]
    jobTitle: Optional[str]
    memberOf: Optional[Array["Organization"]]
    telephoneNumbers: Optional[Array[str]]

    def __init__(
        self,
        address: Optional[str] = None,
        affiliations: Optional[Array["Organization"]] = None,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        emails: Optional[Array[str]] = None,
        familyNames: Optional[Array[str]] = None,
        funders: Optional[Array[Union["Organization", "Person"]]] = None,
        givenNames: Optional[Array[str]] = None,
        honorificPrefix: Optional[str] = None,
        honorificSuffix: Optional[str] = None,
        id: Optional[str] = None,
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
