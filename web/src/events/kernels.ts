import { FileFormatUtils } from '@stencila/components'

type DiscoverExecutableLanguagesEvent = CustomEvent<{
  kernels: string[]
  executableLanguages: FileFormatUtils.FileFormatMap
}>

const kernelsToFileFormatMap = (
  kernels: string[]
): FileFormatUtils.FileFormatMap =>
  kernels.reduce((formats: FileFormatUtils.FileFormatMap, kernelName) => {
    const foundFormat = FileFormatUtils.lookupFormat(kernelName)
    return { ...formats, [foundFormat.name]: foundFormat }
  }, {})

/**
 * Emit a CustomEvent with the name of`'stencila-discover-kernels'` detailing
 * available executable kernels. Custom WebComponents listen to this event and
 * update the list of executable languages in the language selector fields.
 *
 * We also update the global `window.stencilaWebClient.executableLanguages` field
 * so that any components added after the event has fired can be instantiated with
 * an up to date list of languages.
 */
export const onDiscoverExecutableLanguages = (
  kernels: string[]
): {
  kernels: string[]
  executableLanguages: FileFormatUtils.FileFormatMap
} => {
  const executableLanguages = kernelsToFileFormatMap(kernels)
  const event: DiscoverExecutableLanguagesEvent = new CustomEvent(
    'stencila-discover-kernels',
    {
      detail: {
        kernels,
        executableLanguages,
      },
    }
  )

  if (window.stencilaWebClient) {
    window.stencilaWebClient.executableLanguages = executableLanguages
  }

  window.dispatchEvent(event)

  return {
    kernels,
    executableLanguages,
  }
}
