import { JSONSchema7 } from 'json-schema'
import { ConfigSchema } from './types'

type ObjectConfig = JSONSchema7 & { type: 'object' }

export const objectGuard = (schema: ConfigSchema): schema is ObjectConfig =>
  typeof schema !== 'boolean' && schema.type === 'object' && !schema.$schema
