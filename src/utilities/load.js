// Utilities related to dynamic loading of resources

// Dynamic loading of scripts
function script (source, callback) {
  var head = document.getElementsByTagName('head')[0]
  var script = document.createElement('script')
  script.type = 'text/javascript'
  script.src = (window.StencilaHost || '') + source
  if (callback) script.onload = callback
  head.appendChild(script)
}

export default {
  script: script
}
