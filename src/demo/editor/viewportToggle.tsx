import React from 'react'
import { getPreview } from '../utils/preview'

// Change preview iframe's size to simulate a mobile view
const mobileView = (e: React.MouseEvent<HTMLButtonElement>): void => {
  e.preventDefault()
  const preview = getPreview()
  if (preview != null) {
    preview.classList.add('mobile')
  }
}

// Make preview iframe full width
const desktopView = (e: React.MouseEvent<HTMLButtonElement>): void => {
  const preview = getPreview()
  e.preventDefault()
  if (preview != null) {
    preview.classList.remove('mobile')
  }
}

export const ViewportToggle = (): JSX.Element => (
  <>
    <button onClick={mobileView}>Mobile</button>
    <button onClick={desktopView}>Desktop</button>
  </>
)
