/**
 * Lifecycle events for client-side navigation
 *
 * These events allow components and application code to hook into
 * the navigation lifecycle for custom behaviors.
 */

import type {
  GlideEventDetail,
  GlideErrorEventDetail,
  PrefetchEndEventDetail,
  PrefetchStartEventDetail,
} from './types'

/**
 * Navigation event names
 */
export const GlideEvents = {
  /** Fired when navigation starts (before fetch) */
  START: 'stencila:glide:start',

  /** Fired just before DOM swap occurs */
  BEFORE_SWAP: 'stencila:glide:before-swap',

  /** Fired immediately after DOM swap completes */
  AFTER_SWAP: 'stencila:glide:after-swap',

  /** Fired when navigation completes successfully */
  END: 'stencila:glide:end',

  /** Fired when navigation fails */
  ERROR: 'stencila:glide:error',
} as const

/**
 * Prefetch event names
 */
export const PrefetchEvents = {
  /** Fired when prefetch starts */
  START: 'stencila:prefetch:start',

  /** Fired when prefetch completes */
  END: 'stencila:prefetch:end',
} as const

/**
 * Request navigation event (for programmatic navigation)
 */
export const GLIDE_REQUEST = 'stencila:glide' as const

/**
 * Dispatch a navigation lifecycle event
 *
 * @param name - Event name from GlideEvents or PrefetchEvents
 * @param detail - Event detail payload
 * @param cancelable - If true, returns whether the event was NOT cancelled
 * @returns true if event proceeded (or not cancelable), false if cancelled
 */
export function dispatch<T>(name: string, detail: T, cancelable = false): boolean {
  const event = new CustomEvent(name, {
    detail,
    bubbles: true,
    cancelable,
  })
  window.dispatchEvent(event)
  return !event.defaultPrevented
}

/**
 * Type declarations for custom event maps
 */
declare global {
  interface WindowEventMap {
    [GlideEvents.START]: CustomEvent<GlideEventDetail>
    [GlideEvents.BEFORE_SWAP]: CustomEvent<GlideEventDetail>
    [GlideEvents.AFTER_SWAP]: CustomEvent<GlideEventDetail>
    [GlideEvents.END]: CustomEvent<GlideEventDetail>
    [GlideEvents.ERROR]: CustomEvent<GlideErrorEventDetail>
    [PrefetchEvents.START]: CustomEvent<PrefetchStartEventDetail>
    [PrefetchEvents.END]: CustomEvent<PrefetchEndEventDetail>
    [GLIDE_REQUEST]: CustomEvent<{ url: string; trigger: string }>
  }
}
