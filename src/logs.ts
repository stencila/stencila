/**
 * Set up a Winston logger using winston-cfg, which by default reads from config/default.json
 */

import { winstonCfg } from 'winston-cfg'

export const logger = winstonCfg()
