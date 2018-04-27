import StencilaArchive from '../../src/StencilaArchive'
import { _initStencilaArchive } from '../../src/stencilaAppHelpers'

export default function loadRawArchive(rawArchive, context) {
  let archive = new StencilaArchive({}, {}, context)
  archive._sessions = archive._ingest(rawArchive)
  archive._upstreamArchive = rawArchive
  _initStencilaArchive(archive, context)
  return archive
}
