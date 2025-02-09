/**
 * function to provide the correct links to the doc viewer pagination, so they go in order of the menu
 */
export default function pagination() {
    // Select all links in the sidebar
    const sidebar = document.querySelector('#sidebar')
    if (sidebar) {
        const links = Array.from(sidebar.querySelectorAll("a"));
        const currentSlug = window.location.pathname; // Get the current URL path
        
        console.log('links', links, currentSlug)
        let currentIndex = links.findIndex(link => link.classList.contains('active'));

        if (currentIndex !== -1) {
            // Find previous and next links if they exist
            const prevLink = currentIndex > 0 ? links[currentIndex - 1] : null;
            const nextLink = currentIndex < links.length - 1 ? links[currentIndex + 1] : null;

            // Update navigation buttons
            const prevBtn = document.getElementById("prev-link");
            const nextBtn = document.getElementById("next-link");

            if (prevLink) {
                prevBtn.href = prevLink.getAttribute("href");
                const label = prevBtn.querySelector('h4');
                if (label) {
                    label.textContent = `${prevLink.textContent}`;
                }
                prevBtn.classList.remove("hidden");
            }

            if (nextLink) {
                nextBtn.href = nextLink.getAttribute("href");
                const label = nextBtn.querySelector('h4')
                if (label) {
                    label.textContent = `${nextLink.textContent}`;
                }
                nextBtn.classList.remove("hidden");
            }
        }
    }
}