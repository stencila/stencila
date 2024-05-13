export const getTooltipContent = (count: number, provenance: string) => {
  let tooltipContent = `${count} author${count > 1 ? 's' : ''}`
  if (provenance.startsWith('Mw')) {
    tooltipContent += ', machine written'
  } else if (provenance.startsWith('Hw')) {
    tooltipContent += ', human written'
  }
  if (provenance.includes('Me')) {
    tooltipContent += ', machine edited'
  } else if (provenance.includes('He')) {
    tooltipContent += ', human edited'
  }
  if (provenance.includes('Mv')) {
    tooltipContent += ', machine verified'
  } else if (provenance.includes('Hv')) {
    tooltipContent += ', human verified'
  }
  return tooltipContent
}
