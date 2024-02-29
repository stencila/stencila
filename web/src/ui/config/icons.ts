/**
 * Define the acceptable icon values that are available to the config screen.
 */

export type ICON_KEYS =
  | 'ANTHROPIC_API_KEY'
  | 'GOOGLE_AI_API_KEY'
  | 'OPENAI_API_KEY'
  | 'OLLAMA_API_KEY'
  | 'MISTRAL_API_KEY'

export const API_ICONS = {
  ANTHROPIC_API_KEY: 'anthropic',
  GOOGLE_AI_API_KEY: 'google',
  OPENAI_API_KEY: 'open-ai',
  OLLAMA_API_KEY: 'ollama',
  MISTRAL_API_KEY: 'mistral',
} as const
