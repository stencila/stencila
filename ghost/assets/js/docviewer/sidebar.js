/**
 * Handles the opening and closing of the docview sidebar on smaller screens
 */
export function sidebar() {
  const sidebarButton = document.getElementById('sidebar-toggle');

  const sidebarElement = document.getElementById('sidebar');
  if(sidebarButton && sidebarElement) {
    sidebarButton.addEventListener('click', () => {
      sidebarElement.classList.toggle('open');
    });

    document.getElementById('sidebar-close').addEventListener('click', () => {
      sidebarElement.classList.toggle('open');
    });
  };
};

/**
 * Handle collapsing of the sidebar menu sections
 */
export function menuCollapse() {
  document.querySelectorAll(".collapse-toggle").forEach(button => {
      const section = button.closest("li").querySelector(".collapsible-section");
      
      const chevron = button.querySelector('.chevron');

      if (section.classList.contains("expand")) {
        chevron.classList.remove('-rotate-90');
      } else {
        chevron.classList.add('-rotate-90');
      };

      button.addEventListener("click", () => {
          section.classList.toggle("expand");
          if (section.classList.contains("expand")) {
            chevron.classList.remove('-rotate-90');
          } else {
            chevron.classList.add('-rotate-90');
          };
      });
  });
};

