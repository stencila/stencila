import { Result } from 'stencila'

export function valueToSuccessResult<V>(
  value?: V,
  errors?: Result['errors']
): Result<V>
export function valueToSuccessResult(
  value?: undefined,
  errors?: Result['errors']
): Result<undefined> {
  return {
    ok: true,
    value,
    errors: errors ?? [],
  }
}
