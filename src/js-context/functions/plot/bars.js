import marks from './marks'

export default function bars (data, ...encodings) {
  return marks(data, 'bar', ...encodings)
}
