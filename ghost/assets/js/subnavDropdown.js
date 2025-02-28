export default function subnavDropdown() {
  const dropdowns = document.querySelectorAll('.subnav-dropdown');

  if (dropdowns.length) {
    Array.from(dropdowns).forEach((element) => {
      const menuContainer = element.querySelector('.subnav-menu-container');
      const menuTrigger = element.querySelector('button.subnav-menu-trigger');
      let isOpen = false;

      menuTrigger.addEventListener('click', function () {
        isOpen = !isOpen;
        menuContainer.classList.toggle('open')
      });

      document.addEventListener('click', function (event) {
        if (menuContainer.classList.contains('open')) {
            if (!menuTrigger.contains(event.target) && !menuContainer.contains(event.target)) {
                isOpen = false;
                menuContainer.classList.remove('open');
            }
        }
      });
    })
  }
};
