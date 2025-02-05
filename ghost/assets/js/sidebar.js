/**
 * Handles the opening and closing of the docview sidebar on smaller screens
 */
export function sidebar() {
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

/**
 * Handle collapsing of the sidebar menu sections
 */
export function menuCollapse() {
  document.querySelectorAll(".collapse-toggle").forEach(button => {
      const section = button.closest("li").querySelector(".collapsible-section");

      button.addEventListener("click", () => {
          section.classList.toggle("collapsed");
          const chevron = button.querySelector('.chevron')
          if (section.classList.contains("collapsed")) {
            chevron.classList.add('rotate-90')
          } else {
            chevron.classList.remove('rotate-90')
          }
      });
  });
};

