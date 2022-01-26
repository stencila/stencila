import { toMatchFile } from 'jest-file-snapshot'

/**
 * Add https://github.com/satya164/jest-file-snapshot
 *
 * > Jest matcher to write snapshots to a separate file instead of the
 * default snapshot file used by Jest. Writing a snapshot to a separate
 * file means you have proper syntax highlighting in the output file,
 * and better readability without those pesky escape characters.
 */
expect.extend({ toMatchFile })
