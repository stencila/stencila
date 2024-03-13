import { PropertyValues } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { Entity } from './entity'

type CharacterStats = {
  words: number
  characters: number
  charactersExcludingSpaces: number
}

/**
 * Character Stats
 *
 * This component extends entity to add some character stats to an existing
 * entity. This comes in handy in the info view when rendering info about
 * different blocks.
 */
@customElement('stencila-character-stats-entity')
export class CharacterStatsEntity extends Entity {
  /**
   * When this entity is updated, update the information we have about
   * the number of words and characters we have.
   */
  @state()
  protected characterStats: CharacterStats | undefined = undefined

  protected override updated(changedProperties: PropertyValues<this>): void {
    super.updated(changedProperties)

    if (this.characterStats !== undefined) {
      return
    }

    const slots = this.shadowRoot.querySelectorAll('slot')
    let words = 0
    let characters = 0
    let charactersExcludingSpaces = 0

    for (const slot of slots) {
      // TODO - find a better way to include/exclude slots we want as part of
      // our character set.
      if (slot.name === 'authors') {
        continue
      }

      const elements = slot.assignedElements({ flatten: true })

      for (const el of elements) {
        const elementWords = el.textContent.trim().split(/\s+/)
        characters += el.textContent.length
        words += elementWords.length
        charactersExcludingSpaces += elementWords.join().length
      }
    }

    this.characterStats = {
      ...this.characterStats,
      characters: characters,
      words,
      charactersExcludingSpaces,
    }
  }
}
