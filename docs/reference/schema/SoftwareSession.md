# Software Session

**Definition of a compute session, including its software and compute resource requirements and status.**

Usually used to define the environment that is required to execute an executable document. This is "inspired" by the [OCI Runtime Config Schema](https://github.com/opencontainers/runtime-spec/blob/master/schema/config-schema.json) There are a number of properties that did not seem relevant to our use as well as some references that did not seem to be required. We try to stay consistent with the naming used but have dereferenced and combined the config schema and [Linux Schema](https://github.com/opencontainers/runtime-spec/blob/master/schema/defs-linux.json) to use the cpu and memory limits.

This schema type is marked as **experimental** 🧪 and is subject to change.

## Properties

| Name                   | `@id`                                                                                     | Type                                                                                                 | Description                                                               | Inherited from                        |
| ---------------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------- |
| alternateNames         | [schema:alternateName](https://schema.org/alternateName)                                  | Array of string                                                                                      | Alternate names (aliases) for the item.                                   | [Thing](Thing.md)                     |
| clientsLimit           | [stencila:clientsLimit](https://schema.stenci.la/clientsLimit.jsonld)                     | number                                                                                               | The maximum number of concurrent clients the session is limited to.       | [SoftwareSession](SoftwareSession.md) |
| clientsRequest         | [stencila:clientsRequest](https://schema.stenci.la/clientsRequest.jsonld)                 | number                                                                                               | The maximum number of concurrent clients requested for the session.       | [SoftwareSession](SoftwareSession.md) |
| cpuLimit               | [stencila:cpuLimit](https://schema.stenci.la/cpuLimit.jsonld)                             | number                                                                                               | The amount of CPU the session is limited to.                              | [SoftwareSession](SoftwareSession.md) |
| cpuRequest             | [stencila:cpuRequest](https://schema.stenci.la/cpuRequest.jsonld)                         | number                                                                                               | The amount of CPU requested for the session.                              | [SoftwareSession](SoftwareSession.md) |
| dateEnd                | [schema:endDate](https://schema.org/endDate)                                              | [Date](Date.md)                                                                                      | The date-time that the session ended.                                     | [SoftwareSession](SoftwareSession.md) |
| dateStart              | [schema:startDate](https://schema.org/startDate)                                          | [Date](Date.md)                                                                                      | The date-time that the session began.                                     | [SoftwareSession](SoftwareSession.md) |
| description            | [schema:description](https://schema.org/description)                                      | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md) _or_ string | A description of the item. See note [1](#notes).                          | [Thing](Thing.md)                     |
| durationLimit          | [stencila:durationLimit](https://schema.stenci.la/durationLimit.jsonld)                   | number                                                                                               | The maximum duration (seconds) the session is limited to.                 | [SoftwareSession](SoftwareSession.md) |
| durationRequest        | [stencila:durationRequest](https://schema.stenci.la/durationRequest.jsonld)               | number                                                                                               | The maximum duration (seconds) requested for the session.                 | [SoftwareSession](SoftwareSession.md) |
| environment            | [stencila:environment](https://schema.stenci.la/environment.jsonld)                       | [SoftwareEnvironment](SoftwareEnvironment.md)                                                        | The software environment to execute this session in.                      | [SoftwareSession](SoftwareSession.md) |
| id                     | [schema:id](https://schema.org/id)                                                        | string                                                                                               | The identifier for this item.                                             | [Entity](Entity.md)                   |
| identifiers            | [schema:identifier](https://schema.org/identifier)                                        | Array of ([PropertyValue](PropertyValue.md) _or_ string)                                             | Any kind of identifier for any kind of Thing. See note [2](#notes).       | [Thing](Thing.md)                     |
| images                 | [schema:image](https://schema.org/image)                                                  | Array of ([ImageObject](ImageObject.md) _or_ Format 'uri')                                           | Images of the item.                                                       | [Thing](Thing.md)                     |
| memoryLimit            | [stencila:memoryLimit](https://schema.stenci.la/memoryLimit.jsonld)                       | number                                                                                               | The amount of memory that the session is limited to.                      | [SoftwareSession](SoftwareSession.md) |
| memoryRequest          | [stencila:memoryRequest](https://schema.stenci.la/memoryRequest.jsonld)                   | number                                                                                               | The amount of memory requested for the session.                           | [SoftwareSession](SoftwareSession.md) |
| meta                   | [stencila:meta](https://schema.stenci.la/meta.jsonld)                                     | object                                                                                               | Metadata associated with this item.                                       | [Entity](Entity.md)                   |
| name                   | [schema:name](https://schema.org/name)                                                    | string                                                                                               | The name of the item.                                                     | [Thing](Thing.md)                     |
| networkTransferLimit   | [stencila:networkTransferLimit](https://schema.stenci.la/networkTransferLimit.jsonld)     | number                                                                                               | The amount of network data transfer (GiB) that the session is limited to. | [SoftwareSession](SoftwareSession.md) |
| networkTransferRequest | [stencila:networkTransferRequest](https://schema.stenci.la/networkTransferRequest.jsonld) | number                                                                                               | The amount of network data transfer (GiB) requested for the session.      | [SoftwareSession](SoftwareSession.md) |
| status                 | [stencila:sessionStatus](https://schema.stenci.la/sessionStatus.jsonld)                   | 'Unknown', 'Starting', 'Started', 'Stopping', 'Stopped', 'Failed'                                    | The status of the session (starting, stopped, etc).                       | [SoftwareSession](SoftwareSession.md) |
| timeoutLimit           | [stencila:timeoutLimit](https://schema.stenci.la/timeoutLimit.jsonld)                     | number                                                                                               | The inactivity timeout (seconds) the session is limited to.               | [SoftwareSession](SoftwareSession.md) |
| timeoutRequest         | [stencila:timeoutRequest](https://schema.stenci.la/timeoutRequest.jsonld)                 | number                                                                                               | The inactivity timeout (seconds) requested for the session.               | [SoftwareSession](SoftwareSession.md) |
| url                    | [schema:url](https://schema.org/url)                                                      | Format 'uri'                                                                                         | The URL of the item.                                                      | [Thing](Thing.md)                     |
| volumeMounts           | [stencila:volumeMounts](https://schema.stenci.la/volumeMounts.jsonld)                     | Array of [VolumeMount](VolumeMount.md)                                                               | Volumes to mount in the session.                                          | [SoftwareSession](SoftwareSession.md) |

## Notes

1. **description** : Allows for the description to be an array of nodes (e.g. an array of inline content, or a couple of paragraphs), or a string. The `minItems` restriction avoids a string being coerced into an array with a single string item.
2. **identifiers** : Some identifiers have specific properties e.g the `issn` property for the `Periodical` type. These should be used in preference to this property which is intended for identifiers that do not yet have a specific property. Identifiers can be represented as strings, but using a `PropertyValue` will usually be better because it allows for `propertyID` (i.e. the type of identifier).

## Related

- Parent: [Thing](Thing.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/SoftwareSession.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/SoftwareSession.schema.json)
- Python [`class SoftwareSession`](https://stencila.github.io/schema/python/docs/types.html#schema.types.SoftwareSession)
- TypeScript [`interface SoftwareSession`](https://stencila.github.io/schema/ts/docs/interfaces/softwaresession.html)
- R [`class SoftwareSession`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct SoftwareSession`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.SoftwareSession.html)

## Source

This documentation was generated from [SoftwareSession.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/SoftwareSession.schema.yaml).
