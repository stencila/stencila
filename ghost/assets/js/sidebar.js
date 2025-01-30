
export default function sidebar () {
  const sidebarButton = document.getElementById('sidebar-toggle')

  const sidebarElement = document.getElementById('sidebar')

  sidebarButton.addEventListener('click', (e) => {
    sidebarElement.classList.toggle('open')
  })

  document.getElementById('sidebar-close').addEventListener('click', (e) => {
    console.log('hi')
    sidebarElement.classList.toggle('open')
  })
}

