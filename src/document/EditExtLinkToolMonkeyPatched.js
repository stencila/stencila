import { ToggleTool } from 'substance'
import FunctionUsageComponent from '../shared/FunctionUsageComponent'

export default class EditExtLinkToolMonkeyPatched extends ToggleTool {
  render($$) {
    let el = $$('div').addClass('sc-edit-ext-link-tool-monkey-patched').append(
      $$(FunctionUsageComponent, {
        spec: {
          name: 'sum',
          summary: 'Returns the sum of a range',
          examples: [
            'sum(A1:A5)'
          ],
          params: [
            { name: 'range', type: 'range', description: 'A range (array of numbers) to be summed up' }
          ],
          returns: { type: 'number', description: 'The sum of numbers in the given range'}
        },
        paramIndex: 0
      })
    )
    return el
  }
}
