export default function mobileMenu() {
    const menuTrigger = document.getElementById("mobile-menu-trigger");
    const menuContainer = document.getElementById("mobile-menu-container");
    const burgerIcon = document.getElementById("burger-icon");
    const chevronIcon = document.getElementById("chevron-icon");

    let isOpen = false;

    menuTrigger.addEventListener("click", function () {
        isOpen = !isOpen;

        // Toggle menu animation
        menuContainer.classList.toggle('open')

        // Toggle icon animations
        burgerIcon.classList.toggle("hidden");
        chevronIcon.classList.toggle("hidden");
        chevronIcon.classList.toggle("rotate-180");
    });

    // Close menu when clicking outside
    document.addEventListener("click", function (event) {
        if (menuContainer.classList.contains('open')) {
            if (!menuTrigger.contains(event.target) && !menuContainer.contains(event.target)) {
                isOpen = false;
                menuContainer.classList.add("opacity-0", "scale-y-0");
                menuContainer.classList.remove("opacity-100", "scale-y-100");
                burgerIcon.classList.remove("hidden");
                chevronIcon.classList.add("hidden");
            }
        }
    });
}