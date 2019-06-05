import * as winston from 'winston'

/**
 * Set up a Winston logger.
 *
 * For now, just log to console, until various bugs are fixed to allow configuration from file.
 */
export function setupLogger(configPath?: string): winston.Logger {
  return winston.createLogger({
    format: winston.format.simple(),
    transports: [new winston.transports.Console({ level: 'debug' })]
  })
}
