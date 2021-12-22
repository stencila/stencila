import {
  DiscoverExecutableLanguagesEvent,
  FileFormatUtils,
} from '@stencila/components'

const languagesToFileFormatMap = (
  kernels: string[]
): FileFormatUtils.FileFormatMap =>
  kernels.reduce((formats: FileFormatUtils.FileFormatMap, kernelName) => {
    const foundFormat = FileFormatUtils.lookupFormat(kernelName)
    return { ...formats, [foundFormat.name]: foundFormat }
  }, {})

/**
 * Emit a CustomEvent with the name of `stencila-discover-executable-languages` detailing
 * available executable languages. Custom WebComponents listen to this event and
 * update the list of executable languages in the language selector fields.
 *
 * We also update the global `window.stencilaWebClient.executableLanguages` field
 * so that any components added after the event has fired can be instantiated with
 * an up to date list of languages.
 */
export const onDiscoverExecutableLanguages = (
  languageNames: string[]
): {
  languages: FileFormatUtils.FileFormatMap
} => {
  const executableLanguages = languagesToFileFormatMap(languageNames)
  const event: DiscoverExecutableLanguagesEvent = new CustomEvent(
    'stencila-discover-executable-languages',
    {
      detail: {
        languages: executableLanguages,
      },
    }
  )

  window.stencilaWebClient.executableLanguages = executableLanguages

  window.dispatchEvent(event)

  return {
    languages: executableLanguages,
  }
}
