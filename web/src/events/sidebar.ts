import { SidebarContext } from '../contexts/sidebar-context'
import { MainContextEvent } from '../types'

/**
 * Create an event to dispatch to manipulate the sidebar context.
 *
 * @template T
 * @param {MainContextEvent} name
 * @param {Pick<SidebarContext, T>} [detail]
 */
export const emitSidebarEvent = <T extends keyof SidebarContext>(
  name: MainContextEvent,
  detail?: Pick<SidebarContext, T>
): CustomEvent => {
  return new CustomEvent(name, {
    bubbles: true,
    composed: true,
    detail,
  })
}
