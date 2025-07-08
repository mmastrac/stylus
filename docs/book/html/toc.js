// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="getting-started/overview.html"><strong aria-hidden="true">1.</strong> Overview</a></li><li class="chapter-item expanded "><a href="getting-started/running.html"><strong aria-hidden="true">2.</strong> Running Stylus</a></li><li><ol class="section"><li class="chapter-item expanded "><div><strong aria-hidden="true">2.1.</strong> stylus init</div></li><li class="chapter-item expanded "><div><strong aria-hidden="true">2.2.</strong> stylus test</div></li><li class="chapter-item expanded "><div><strong aria-hidden="true">2.3.</strong> stylus run</div></li></ol></li><li class="chapter-item expanded "><a href="getting-started/creating-project.html"><strong aria-hidden="true">3.</strong> Creating a Stylus Project</a></li><li class="chapter-item expanded "><a href="getting-started/creating-monitors.html"><strong aria-hidden="true">4.</strong> Creating Monitors</a></li><li class="chapter-item expanded "><a href="getting-started/creating-pages.html"><strong aria-hidden="true">5.</strong> Creating Monitor Pages</a></li><li class="chapter-item expanded affix "><li class="part-title">Configuration</li><li class="chapter-item expanded "><a href="configuration/server/index.html"><strong aria-hidden="true">6.</strong> Server Configuration</a></li><li class="chapter-item expanded "><a href="configuration/css/index.html"><strong aria-hidden="true">7.</strong> CSS Configuration</a></li><li class="chapter-item expanded "><a href="configuration/monitor/index.html"><strong aria-hidden="true">8.</strong> Monitor Configuration</a></li><li class="chapter-item expanded "><a href="configuration/advanced.html"><strong aria-hidden="true">9.</strong> Advanced Configuration</a></li><li class="chapter-item expanded affix "><li class="part-title">Examples</li><li class="chapter-item expanded "><a href="examples/general/index.html"><strong aria-hidden="true">10.</strong> General Tips</a></li><li class="chapter-item expanded "><a href="examples/ping/index.html"><strong aria-hidden="true">11.</strong> Ping Monitoring</a></li><li class="chapter-item expanded "><a href="examples/ssh/index.html"><strong aria-hidden="true">12.</strong> SSH Monitoring</a></li><li class="chapter-item expanded "><a href="examples/snmp/index.html"><strong aria-hidden="true">13.</strong> SNMP Monitoring</a></li><li class="chapter-item expanded "><a href="examples/scraping/index.html"><strong aria-hidden="true">14.</strong> HTML/API Scraping</a></li><li class="chapter-item expanded affix "><li class="part-title">Tutorials</li><li class="chapter-item expanded "><a href="tutorials/svg-diagrams.html"><strong aria-hidden="true">15.</strong> Creating SVG Diagrams</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
