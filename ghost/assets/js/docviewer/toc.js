
const OFFSET = 160


/**
 * Keeps the fixed menu aligned with the right hand side of the page container
 * @returns 
 */
function adjustTocPosition(toc, pageContainer) {
  if (!pageContainer) {
    return
  }

  if (!toc || !pageContainer) return;

  const containerRect = pageContainer.getBoundingClientRect();
  const viewportWidth = window.innerWidth;

  if (viewportWidth > 1200) { // Adjust based on your layout
      const offsetRight = viewportWidth - (containerRect.left + containerRect.width);
      toc.style.right = `${offsetRight}px`;
  } else {
      toc.style.right = "0px"; // Default to edge if screen is too small
  }
}

/**
 * Adds active class to the link matching the heading closest to the top of the page
 */
function highlightActiveHeading(headings, links) {
  let closestHeading = null;
  let minDistance = Infinity;
  
  headings.forEach((heading) => {
      const rect = heading.getBoundingClientRect();
      const distance = Math.abs(rect.top - OFFSET);
      
      if (rect.top >= 0 && distance < minDistance) {
          minDistance = distance;
          closestHeading = heading;
      }
  });
  
  links.forEach((link, heading) => {
      if (heading === closestHeading) {
          link.classList.add('active');
      } else {
          link.classList.remove('active');
      }
  });
}


export default function generateTableOfContents() {
  const article = document.querySelector('#doc-content');
  const tocOuter = document.querySelector('#doc-toc-outer');

  const toc = document.querySelector('#doc-toc-inner');

  const pageContainer = document.querySelector('.page-container')

  if (!article || !toc) {
    return;
  }
  
  const headings = article.querySelectorAll('h1, h2, h3');
  const tocList = document.createElement('ul');
  tocList.id = 'toc-inner';

  let lastLevel = 1;
  let listStack = [tocList];

  let tocLinks = new Map();

  if (headings.length > 0) {
    headings.forEach((heading, index) => {
        const level = parseInt(heading.tagName[1]);
        const item = document.createElement('li');
        const link = document.createElement('a');

        // Generate an ID if the heading doesn’t have one
        if (!heading.id) {
            heading.id = `toc-heading-${index}`;
        }
        
        link.href = `#`;
        link.textContent = heading.textContent;

        link.addEventListener('click', (e) => {
          e.preventDefault()
          const targetPosition = heading.getBoundingClientRect().top + window.scrollY - OFFSET;
          window.scrollTo({ top: targetPosition, behavior: 'smooth'})
        })

        item.appendChild(link);
        tocLinks.set(heading, link);

        if (level > lastLevel) {
            // Create a new sublist and push it to the stack
            const newList = document.createElement('ul');
            listStack[listStack.length - 1].lastElementChild?.appendChild(newList);
            listStack.push(newList);
        } else if (level < lastLevel) {
            // Pop to the correct level
            while (listStack.length > (level - 1)) {
                listStack.pop();
            }
        }

        // Append item to the current last list
        listStack[listStack.length - 1].appendChild(item);
        lastLevel = level;
    });

    toc.innerHTML = ''
    toc.appendChild(tocList);
  }
  

  adjustTocPosition(tocOuter, pageContainer)
  highlightActiveHeading(headings, tocLinks)
  window.addEventListener('resize', () => adjustTocPosition(tocOuter, pageContainer))
  window.addEventListener('scroll', () => highlightActiveHeading(headings, tocLinks))
}