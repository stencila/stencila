import SnapshotEngineBase from 'substance/collab/SnapshotEngine'

/**
 * Handles computation of snapshots for Stencila component sessions.
 *
 * This extends `substance/collab/SnapshotEngine` to handle the creation of
 * alternative document model types (at time of writing this base class seemed
 * to be able to only create one type of document based on a configurator)
 *
 * @class      SnapshotEngine (name)
 * @param      {<type>}  config  The configuration
 */
class SnapshotEngine extends SnapshotEngineBase {

  constructor (config) {
    super(config)

    this.modelFactory = config.modelFactory
  }

  _createDocumentInstance (schemaName) {
    return this.modelFactory.createDocument(schemaName)
  }
}

export default SnapshotEngine
