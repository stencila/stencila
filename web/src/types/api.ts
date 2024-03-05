/**
 * Secret - returned in an array from the API.
 */
export type Secret = {
  category: 'AiApiKey'
  name: string
  title: string
  description: string
  redacted?: string
}
