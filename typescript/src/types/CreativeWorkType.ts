// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Article } from './Article'
import { AudioObject } from './AudioObject'
import { Claim } from './Claim'
import { Collection } from './Collection'
import { Comment } from './Comment'
import { Datatable } from './Datatable'
import { Directory } from './Directory'
import { Figure } from './Figure'
import { File } from './File'
import { ImageObject } from './ImageObject'
import { MediaObject } from './MediaObject'
import { Periodical } from './Periodical'
import { PublicationIssue } from './PublicationIssue'
import { PublicationVolume } from './PublicationVolume'
import { Review } from './Review'
import { SoftwareApplication } from './SoftwareApplication'
import { SoftwareSourceCode } from './SoftwareSourceCode'
import { Table } from './Table'
import { VideoObject } from './VideoObject'

// Union type for all types that are descended from `CreativeWork`
export type CreativeWorkType =
  Article |
  AudioObject |
  Claim |
  Collection |
  Comment |
  Datatable |
  Directory |
  Figure |
  File |
  ImageObject |
  MediaObject |
  Periodical |
  PublicationIssue |
  PublicationVolume |
  Review |
  SoftwareApplication |
  SoftwareSourceCode |
  Table |
  VideoObject;
