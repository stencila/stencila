from typing import Any, Dict, List as Array, Optional, Union
from enum import Enum

Enum0 = Enum("0", ["ascending", "descending", "unordered"])


class Entity:
    """The most basic item, defining the minimum properties required."""

    id: Optional[str]
    meta: Optional[Dict[str, Any]]

    def __init__(
        self,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(

        )
        if id is not None:
            self.id = id
        if meta is not None:
            self.meta = meta


class DatatableColumnSchema(Entity):
    items: Dict[str, Any]
    uniqueItems: Optional[bool]

    def __init__(
        self,
        items: Dict[str, Any],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        uniqueItems: Optional[bool] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if items is not None:
            self.items = items
        if uniqueItems is not None:
            self.uniqueItems = uniqueItems


class Mark(Entity):
    """
    A base class for nodes that mark some other inline content (e.g. `string`
    or other `InlineContent` nodes) in some way (e.g. as being emphasised, or
    quoted).
    """

    content: Array["InlineContent"]

    def __init__(
        self,
        content: Array["InlineContent"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if content is not None:
            self.content = content


class Delete(Mark):
    """Content that is marked for deletion"""

    def __init__(
        self,
        content: Array["InlineContent"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            content=content,
            id=id,
            meta=meta
        )



class Emphasis(Mark):
    """Emphasised content."""

    def __init__(
        self,
        content: Array["InlineContent"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            content=content,
            id=id,
            meta=meta
        )



class Thing(Entity):
    """The most generic type of item https://schema.org/Thing."""

    alternateNames: Optional[Array[str]]
    description: Optional[str]
    name: Optional[str]
    url: Optional[str]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        url: Optional[str] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if alternateNames is not None:
            self.alternateNames = alternateNames
        if description is not None:
            self.description = description
        if name is not None:
            self.name = name
        if url is not None:
            self.url = url


class Brand(Thing):
    """
    A brand is a name used by an organization or business person for labeling a
    product, product group, or similar. https://schema.org/Brand.
    """

    logo: Optional[Union[str, "ImageObject"]]
    reviews: Optional[Array[str]]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        logo: Optional[Union[str, "ImageObject"]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        reviews: Optional[Array[str]] = None,
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
        if logo is not None:
            self.logo = logo
        if reviews is not None:
            self.reviews = reviews


class Code(Thing):
    """Inline code."""

    value: str
    language: Optional[str]

    def __init__(
        self,
        value: str,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        language: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
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
        if value is not None:
            self.value = value
        if language is not None:
            self.language = language


class CodeBlock(Code):
    """A code block."""

    def __init__(
        self,
        value: str,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        language: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        url: Optional[str] = None
    ) -> None:
        super().__init__(
            value=value,
            alternateNames=alternateNames,
            description=description,
            id=id,
            language=language,
            meta=meta,
            name=name,
            url=url
        )



class ContactPoint(Thing):
    """
    A contact point—for example, a R&D department.
    https://schema.org/ContactPoint.
    """

    availableLanguages: Optional[Array[str]]
    emails: Optional[Array[str]]
    telephone: Optional[str]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        availableLanguages: Optional[Array[str]] = None,
        description: Optional[str] = None,
        emails: Optional[Array[str]] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        telephone: Optional[str] = None,
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
        if availableLanguages is not None:
            self.availableLanguages = availableLanguages
        if emails is not None:
            self.emails = emails
        if telephone is not None:
            self.telephone = telephone


class CreativeWork(Thing):
    """
    The most generic kind of creative work, including books, movies,
    photographs, software programs, etc. https://schema.org/CreativeWork
    """

    authors: Optional[Array[Union["Person", "Organization"]]]
    citations: Optional[Array[Union[str, "CreativeWork"]]]
    content: Optional[Array["Node"]]
    dateCreated: Optional[str]
    dateModified: Optional[str]
    datePublished: Optional[str]
    editors: Optional[Array["Person"]]
    funders: Optional[Array[Union["Person", "Organization"]]]
    isPartOf: Optional["CreativeWork"]
    licenses: Optional[Array[Union[str, "CreativeWork"]]]
    parts: Optional[Array["CreativeWork"]]
    publisher: Optional[Union["Person", "Organization"]]
    text: Optional[str]
    title: Optional[str]
    version: Optional[Union[str, float]]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            description=description,
            id=id,
            meta=meta,
            name=name,
            url=url
        )
        if authors is not None:
            self.authors = authors
        if citations is not None:
            self.citations = citations
        if content is not None:
            self.content = content
        if dateCreated is not None:
            self.dateCreated = dateCreated
        if dateModified is not None:
            self.dateModified = dateModified
        if datePublished is not None:
            self.datePublished = datePublished
        if editors is not None:
            self.editors = editors
        if funders is not None:
            self.funders = funders
        if isPartOf is not None:
            self.isPartOf = isPartOf
        if licenses is not None:
            self.licenses = licenses
        if parts is not None:
            self.parts = parts
        if publisher is not None:
            self.publisher = publisher
        if text is not None:
            self.text = text
        if title is not None:
            self.title = title
        if version is not None:
            self.version = version


class Article(CreativeWork):
    authors: Array[Union["Person", "Organization"]]
    title: str
    environment: Optional["Environment"]

    def __init__(
        self,
        authors: Array[Union["Person", "Organization"]],
        title: str,
        alternateNames: Optional[Array[str]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        environment: Optional["Environment"] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            authors=authors,
            title=title,
            alternateNames=alternateNames,
            citations=citations,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            url=url,
            version=version
        )
        if authors is not None:
            self.authors = authors
        if title is not None:
            self.title = title
        if environment is not None:
            self.environment = environment


class Collection(CreativeWork):
    """A created collection of CreativeWorks or other artefacts."""

    parts: Array["CreativeWork"]

    def __init__(
        self,
        parts: Array["CreativeWork"],
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            parts=parts,
            alternateNames=alternateNames,
            authors=authors,
            citations=citations,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if parts is not None:
            self.parts = parts


class Datatable(CreativeWork):
    """A table of data."""

    columns: Array["DatatableColumn"]

    def __init__(
        self,
        columns: Array["DatatableColumn"],
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            authors=authors,
            citations=citations,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if columns is not None:
            self.columns = columns


class MediaObject(CreativeWork):
    """
    A media object, such as an image, video, or audio object embedded in a web
    page or a downloadable dataset. https://schema.org/MediaObject
    """

    contentUrl: str
    bitrate: Optional[float]
    contentSize: Optional[float]
    embedUrl: Optional[str]
    format: Optional[str]

    def __init__(
        self,
        contentUrl: str,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        bitrate: Optional[float] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        contentSize: Optional[float] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        embedUrl: Optional[str] = None,
        format: Optional[str] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            authors=authors,
            citations=citations,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if contentUrl is not None:
            self.contentUrl = contentUrl
        if bitrate is not None:
            self.bitrate = bitrate
        if contentSize is not None:
            self.contentSize = contentSize
        if embedUrl is not None:
            self.embedUrl = embedUrl
        if format is not None:
            self.format = format


class AudioObject(MediaObject):
    """An audio file. https://schema.org/AudioObject"""

    caption: Optional[str]
    transcript: Optional[str]

    def __init__(
        self,
        contentUrl: str,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        bitrate: Optional[float] = None,
        caption: Optional[str] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        contentSize: Optional[float] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        embedUrl: Optional[str] = None,
        format: Optional[str] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        transcript: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            contentUrl=contentUrl,
            alternateNames=alternateNames,
            authors=authors,
            bitrate=bitrate,
            citations=citations,
            content=content,
            contentSize=contentSize,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            embedUrl=embedUrl,
            format=format,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if caption is not None:
            self.caption = caption
        if transcript is not None:
            self.transcript = transcript


class SoftwareSourceCode(CreativeWork):
    """
    Computer programming source code. Example: Full (compile ready) solutions,
    code snippet samples, scripts, templates.
    """

    codeRepository: Optional[str]
    codeSampleType: Optional[str]
    maintainers: Optional[Array[Union["Organization", "Person"]]]
    programmingLanguage: Optional[str]
    runtimePlatform: Optional[Array[str]]
    softwareRequirements: Optional[Array[Union["SoftwareSourceCode", "SoftwareApplication", str]]]
    targetProducts: Optional[Array["SoftwareApplication"]]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        codeRepository: Optional[str] = None,
        codeSampleType: Optional[str] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        maintainers: Optional[Array[Union["Organization", "Person"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        programmingLanguage: Optional[str] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        runtimePlatform: Optional[Array[str]] = None,
        softwareRequirements: Optional[Array[Union["SoftwareSourceCode", "SoftwareApplication", str]]] = None,
        targetProducts: Optional[Array["SoftwareApplication"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            authors=authors,
            citations=citations,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if codeRepository is not None:
            self.codeRepository = codeRepository
        if codeSampleType is not None:
            self.codeSampleType = codeSampleType
        if maintainers is not None:
            self.maintainers = maintainers
        if programmingLanguage is not None:
            self.programmingLanguage = programmingLanguage
        if runtimePlatform is not None:
            self.runtimePlatform = runtimePlatform
        if softwareRequirements is not None:
            self.softwareRequirements = softwareRequirements
        if targetProducts is not None:
            self.targetProducts = targetProducts


class CodeChunk(SoftwareSourceCode):
    """A executable chunk of code."""

    outputs: Optional[Array["Node"]]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        codeRepository: Optional[str] = None,
        codeSampleType: Optional[str] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        maintainers: Optional[Array[Union["Organization", "Person"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        outputs: Optional[Array["Node"]] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        programmingLanguage: Optional[str] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        runtimePlatform: Optional[Array[str]] = None,
        softwareRequirements: Optional[Array[Union["SoftwareSourceCode", "SoftwareApplication", str]]] = None,
        targetProducts: Optional[Array["SoftwareApplication"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            authors=authors,
            citations=citations,
            codeRepository=codeRepository,
            codeSampleType=codeSampleType,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            maintainers=maintainers,
            meta=meta,
            name=name,
            parts=parts,
            programmingLanguage=programmingLanguage,
            publisher=publisher,
            runtimePlatform=runtimePlatform,
            softwareRequirements=softwareRequirements,
            targetProducts=targetProducts,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if outputs is not None:
            self.outputs = outputs


class CodeExpr(SoftwareSourceCode):
    """An expression."""

    value: Optional["Node"]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        codeRepository: Optional[str] = None,
        codeSampleType: Optional[str] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        maintainers: Optional[Array[Union["Organization", "Person"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        programmingLanguage: Optional[str] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        runtimePlatform: Optional[Array[str]] = None,
        softwareRequirements: Optional[Array[Union["SoftwareSourceCode", "SoftwareApplication", str]]] = None,
        targetProducts: Optional[Array["SoftwareApplication"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        value: Optional["Node"] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            authors=authors,
            citations=citations,
            codeRepository=codeRepository,
            codeSampleType=codeSampleType,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            maintainers=maintainers,
            meta=meta,
            name=name,
            parts=parts,
            programmingLanguage=programmingLanguage,
            publisher=publisher,
            runtimePlatform=runtimePlatform,
            softwareRequirements=softwareRequirements,
            targetProducts=targetProducts,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if value is not None:
            self.value = value


class DatatableColumn(Thing):
    name: str
    values: Array[Any]
    schema: Optional["DatatableColumnSchema"]

    def __init__(
        self,
        name: str,
        values: Array[Any],
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        schema: Optional["DatatableColumnSchema"] = None,
        url: Optional[str] = None
    ) -> None:
        super().__init__(
            name=name,
            alternateNames=alternateNames,
            description=description,
            id=id,
            meta=meta,
            url=url
        )
        if name is not None:
            self.name = name
        if values is not None:
            self.values = values
        if schema is not None:
            self.schema = schema


class Environment(Thing):
    """A computational environment."""

    name: str
    adds: Optional[Array["SoftwareSourceCode"]]
    environmentSource: Optional[str]
    extends: Optional[Array["Environment"]]
    removes: Optional[Array["SoftwareSourceCode"]]

    def __init__(
        self,
        name: str,
        adds: Optional[Array["SoftwareSourceCode"]] = None,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        environmentSource: Optional[str] = None,
        extends: Optional[Array["Environment"]] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        removes: Optional[Array["SoftwareSourceCode"]] = None,
        url: Optional[str] = None
    ) -> None:
        super().__init__(
            name=name,
            alternateNames=alternateNames,
            description=description,
            id=id,
            meta=meta,
            url=url
        )
        if name is not None:
            self.name = name
        if adds is not None:
            self.adds = adds
        if environmentSource is not None:
            self.environmentSource = environmentSource
        if extends is not None:
            self.extends = extends
        if removes is not None:
            self.removes = removes


class Heading(Entity):
    """Heading"""

    content: Array["InlineContent"]
    depth: float

    def __init__(
        self,
        content: Array["InlineContent"],
        depth: float,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if content is not None:
            self.content = content
        if depth is not None:
            self.depth = depth


class ImageObject(MediaObject):
    """An image file. https://schema.org/ImageObject"""

    caption: Optional[str]
    thumbnail: Optional["ImageObject"]

    def __init__(
        self,
        contentUrl: str,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        bitrate: Optional[float] = None,
        caption: Optional[str] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        contentSize: Optional[float] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        embedUrl: Optional[str] = None,
        format: Optional[str] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        thumbnail: Optional["ImageObject"] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            contentUrl=contentUrl,
            alternateNames=alternateNames,
            authors=authors,
            bitrate=bitrate,
            citations=citations,
            content=content,
            contentSize=contentSize,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            embedUrl=embedUrl,
            format=format,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if caption is not None:
            self.caption = caption
        if thumbnail is not None:
            self.thumbnail = thumbnail


class Include(Entity):
    """
    A directive to include content from an external source (e.g. file, URL) or
    content.
    """

    source: str
    content: Optional[Array["Node"]]
    hash: Optional[str]
    mediaType: Optional[str]

    def __init__(
        self,
        source: str,
        content: Optional[Array["Node"]] = None,
        hash: Optional[str] = None,
        id: Optional[str] = None,
        mediaType: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if source is not None:
            self.source = source
        if content is not None:
            self.content = content
        if hash is not None:
            self.hash = hash
        if mediaType is not None:
            self.mediaType = mediaType


class Link(Entity):
    """
    A hyperlink to other pages, sections within the same document, resources,
    or any URL.
    """

    content: Array["InlineContent"]
    target: str
    relation: Optional[str]
    title: Optional[str]

    def __init__(
        self,
        content: Array["InlineContent"],
        target: str,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        relation: Optional[str] = None,
        title: Optional[str] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if content is not None:
            self.content = content
        if target is not None:
            self.target = target
        if relation is not None:
            self.relation = relation
        if title is not None:
            self.title = title


class List(Entity):
    """A list of items."""

    items: Array["ListItem"]
    order: Optional["Enum0"]

    def __init__(
        self,
        items: Array["ListItem"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        order: Optional["Enum0"] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if items is not None:
            self.items = items
        if order is not None:
            self.order = order


class ListItem(Entity):
    """A single item in a list."""

    content: Array["Node"]
    checked: Optional[bool]

    def __init__(
        self,
        content: Array["Node"],
        checked: Optional[bool] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if content is not None:
            self.content = content
        if checked is not None:
            self.checked = checked


class Mount(Thing):
    """Describes a volume mount from a host to container."""

    mountDestination: str
    mountOptions: Optional[Array[str]]
    mountSource: Optional[str]
    mountType: Optional[str]

    def __init__(
        self,
        mountDestination: str,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        mountOptions: Optional[Array[str]] = None,
        mountSource: Optional[str] = None,
        mountType: Optional[str] = None,
        name: Optional[str] = None,
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
        if mountDestination is not None:
            self.mountDestination = mountDestination
        if mountOptions is not None:
            self.mountOptions = mountOptions
        if mountSource is not None:
            self.mountSource = mountSource
        if mountType is not None:
            self.mountType = mountType


class Organization(Thing):
    """
    An organization such as a school, NGO, corporation, club, etc.
    https://schema.org/Organization.
    """

    address: Optional[str]
    brands: Optional[Array["Brand"]]
    contactPoints: Optional[Array["ContactPoint"]]
    departments: Optional[Array["Organization"]]
    funders: Optional[Array[Union["Organization", "Person"]]]
    legalName: Optional[str]
    parentOrganization: Optional["Organization"]

    def __init__(
        self,
        address: Optional[str] = None,
        alternateNames: Optional[Array[str]] = None,
        brands: Optional[Array["Brand"]] = None,
        contactPoints: Optional[Array["ContactPoint"]] = None,
        departments: Optional[Array["Organization"]] = None,
        description: Optional[str] = None,
        funders: Optional[Array[Union["Organization", "Person"]]] = None,
        id: Optional[str] = None,
        legalName: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parentOrganization: Optional["Organization"] = None,
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
        if brands is not None:
            self.brands = brands
        if contactPoints is not None:
            self.contactPoints = contactPoints
        if departments is not None:
            self.departments = departments
        if funders is not None:
            self.funders = funders
        if legalName is not None:
            self.legalName = legalName
        if parentOrganization is not None:
            self.parentOrganization = parentOrganization


class Paragraph(Entity):
    """Paragraph"""

    content: Array["InlineContent"]

    def __init__(
        self,
        content: Array["InlineContent"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if content is not None:
            self.content = content


class Person(Thing):
    """
    A person (alive, dead, undead, or fictional). https://schema.org/Person.
    """

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


class Product(Thing):
    """
    Any offered product or service. For example, a pair of shoes; a concert
    ticket; the rental of a car; a haircut; or an episode of a TV show streamed
    online. https://schema.org/Product
    """

    brand: Optional["Brand"]
    logo: Optional[Union[str, "ImageObject"]]
    productID: Optional[str]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        brand: Optional["Brand"] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        logo: Optional[Union[str, "ImageObject"]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        productID: Optional[str] = None,
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
        if brand is not None:
            self.brand = brand
        if logo is not None:
            self.logo = logo
        if productID is not None:
            self.productID = productID


class Quote(Mark):
    """
    Inline, quoted content. Analagous to,   - HTML [`<q>`
    element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)
  -
    Pandoc
    [`Quoted`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/De
    """

    citation: Optional[str]

    def __init__(
        self,
        content: Array["InlineContent"],
        citation: Optional[str] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            content=content,
            id=id,
            meta=meta
        )
        if citation is not None:
            self.citation = citation


class QuoteBlock(Entity):
    """A section quoted from somewhere else."""

    content: Array["BlockContent"]
    citation: Optional[str]

    def __init__(
        self,
        content: Array["BlockContent"],
        citation: Optional[str] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if content is not None:
            self.content = content
        if citation is not None:
            self.citation = citation


class ResourceParameters(Thing):
    """
    Describes limits or requested amounts for a particular resource (e.g.
    memory or CPU).
    """

    resourceLimit: Optional[float]
    resourceRequested: Optional[float]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        resourceLimit: Optional[float] = None,
        resourceRequested: Optional[float] = None,
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
        if resourceLimit is not None:
            self.resourceLimit = resourceLimit
        if resourceRequested is not None:
            self.resourceRequested = resourceRequested


class SoftwareApplication(CreativeWork):
    """A software application."""

    softwareRequirements: Optional[Array["SoftwareApplication"]]
    softwareVersion: Optional[str]

    def __init__(
        self,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        softwareRequirements: Optional[Array["SoftwareApplication"]] = None,
        softwareVersion: Optional[str] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            authors=authors,
            citations=citations,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if softwareRequirements is not None:
            self.softwareRequirements = softwareRequirements
        if softwareVersion is not None:
            self.softwareVersion = softwareVersion


class SoftwareSession(Thing):
    """
    Represents a runtime session with the resources and image that is required
    by software to execute.
    """

    environment: "Environment"
    cpuResource: Optional["ResourceParameters"]
    memoryResource: Optional["ResourceParameters"]
    volumeMounts: Optional[Array["Mount"]]

    def __init__(
        self,
        environment: "Environment",
        alternateNames: Optional[Array[str]] = None,
        cpuResource: Optional["ResourceParameters"] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        memoryResource: Optional["ResourceParameters"] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        url: Optional[str] = None,
        volumeMounts: Optional[Array["Mount"]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            description=description,
            id=id,
            meta=meta,
            name=name,
            url=url
        )
        if environment is not None:
            self.environment = environment
        if cpuResource is not None:
            self.cpuResource = cpuResource
        if memoryResource is not None:
            self.memoryResource = memoryResource
        if volumeMounts is not None:
            self.volumeMounts = volumeMounts


class Strong(Mark):
    """
    Strongly emphasised content. Analagous to,   - JATS
    [`<bold>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/bold.
    """

    def __init__(
        self,
        content: Array["InlineContent"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            content=content,
            id=id,
            meta=meta
        )



class Subscript(Mark):
    """
    Subscripted content. Analagous to,   - JATS
    [`<sub>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/sub.ht
    """

    def __init__(
        self,
        content: Array["InlineContent"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            content=content,
            id=id,
            meta=meta
        )



class Superscript(Mark):
    """
    Superscripted content. Analagous to,   - JATS
    [`<sup>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/sup.ht
    """

    def __init__(
        self,
        content: Array["InlineContent"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            content=content,
            id=id,
            meta=meta
        )



class Table(CreativeWork):
    """A table."""

    rows: Array["TableRow"]

    def __init__(
        self,
        rows: Array["TableRow"],
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        title: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            alternateNames=alternateNames,
            authors=authors,
            citations=citations,
            content=content,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if rows is not None:
            self.rows = rows


class TableCell(Entity):
    """A cell within a `Table`."""

    content: Array["InlineContent"]
    colspan: Optional[int]
    name: Optional[str]
    rowspan: Optional[int]

    def __init__(
        self,
        content: Array["InlineContent"],
        colspan: Optional[int] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        rowspan: Optional[int] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if content is not None:
            self.content = content
        if colspan is not None:
            self.colspan = colspan
        if name is not None:
            self.name = name
        if rowspan is not None:
            self.rowspan = rowspan


class TableRow(Entity):
    """A row within a Table."""

    cells: Array["TableCell"]

    def __init__(
        self,
        cells: Array["TableCell"],
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )
        if cells is not None:
            self.cells = cells


class ThematicBreak(Entity):
    """
    A thematic break, such as a scene change in a story, a transition to
    another topic, or a new document.
    """

    def __init__(
        self,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None
    ) -> None:
        super().__init__(
            id=id,
            meta=meta
        )



class VideoObject(MediaObject):
    """A video file. https://schema.org/VideoObject"""

    caption: Optional[str]
    thumbnail: Optional["ImageObject"]
    transcript: Optional[str]

    def __init__(
        self,
        contentUrl: str,
        alternateNames: Optional[Array[str]] = None,
        authors: Optional[Array[Union["Person", "Organization"]]] = None,
        bitrate: Optional[float] = None,
        caption: Optional[str] = None,
        citations: Optional[Array[Union[str, "CreativeWork"]]] = None,
        content: Optional[Array["Node"]] = None,
        contentSize: Optional[float] = None,
        dateCreated: Optional[str] = None,
        dateModified: Optional[str] = None,
        datePublished: Optional[str] = None,
        description: Optional[str] = None,
        editors: Optional[Array["Person"]] = None,
        embedUrl: Optional[str] = None,
        format: Optional[str] = None,
        funders: Optional[Array[Union["Person", "Organization"]]] = None,
        id: Optional[str] = None,
        isPartOf: Optional["CreativeWork"] = None,
        licenses: Optional[Array[Union[str, "CreativeWork"]]] = None,
        meta: Optional[Dict[str, Any]] = None,
        name: Optional[str] = None,
        parts: Optional[Array["CreativeWork"]] = None,
        publisher: Optional[Union["Person", "Organization"]] = None,
        text: Optional[str] = None,
        thumbnail: Optional["ImageObject"] = None,
        title: Optional[str] = None,
        transcript: Optional[str] = None,
        url: Optional[str] = None,
        version: Optional[Union[str, float]] = None
    ) -> None:
        super().__init__(
            contentUrl=contentUrl,
            alternateNames=alternateNames,
            authors=authors,
            bitrate=bitrate,
            citations=citations,
            content=content,
            contentSize=contentSize,
            dateCreated=dateCreated,
            dateModified=dateModified,
            datePublished=datePublished,
            description=description,
            editors=editors,
            embedUrl=embedUrl,
            format=format,
            funders=funders,
            id=id,
            isPartOf=isPartOf,
            licenses=licenses,
            meta=meta,
            name=name,
            parts=parts,
            publisher=publisher,
            text=text,
            title=title,
            url=url,
            version=version
        )
        if caption is not None:
            self.caption = caption
        if thumbnail is not None:
            self.thumbnail = thumbnail
        if transcript is not None:
            self.transcript = transcript


"""
Block content.
"""
BlockContent = Union["CodeBlock", "CodeChunk", "Heading", "List", "ListItem", "Paragraph", "QuoteBlock", "Table", "ThematicBreak"]


"""
Inline content.
"""
InlineContent = Union[None, bool, int, float, str, "Emphasis", "Strong", "Delete", "Subscript", "Superscript", "Quote", "Code", "CodeExpr", "Link", "ImageObject"]


"""
Describes a valid value for any node in the tree.
"""
Node = Union[None, bool, float, int, str, Array[Any], Dict[str, Any], "Entity"]

