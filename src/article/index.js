import { EditorPackage as TextureEditorPackage } from 'substance-texture'

// HACK: can this be done with a simple forward instead of subclassing?
// export default class ArticleEditor extends TextureEditorPackage.Editor {}
const Editor = TextureEditorPackage.Editor
export { Editor as ArticleEditor }

export { default as ArticleEditorPackage } from './ArticleEditorPackage'
export { default as ArticleLoader } from './ArticleLoader'
