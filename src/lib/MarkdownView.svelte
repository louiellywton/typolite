<script lang="ts">
  import { onMount, afterUpdate } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  import type { ParsedDocument } from './types';

  export let document: ParsedDocument;
  
  const dispatch = createEventDispatcher<{
    lineScrolled: { line: number };
  }>();

  let contentElement: HTMLElement;
  let isScrolling = false;
  let scrollTimeout: number;

  onMount(() => {
    processContent();
  });

  afterUpdate(() => {
    processContent();
  });

  function processContent() {
    if (!contentElement) return;

    // Process math expressions with KaTeX
    processMathElements();
    
    // Process code highlighting
    processCodeBlocks();
    
    // Process Mermaid diagrams
    processMermaidDiagrams();
    
    // Add anchor links to headings
    processHeadings();
  }

  function processMathElements() {
    const mathElements = contentElement.querySelectorAll('.katex-inline');
    mathElements.forEach((element: Element) => {
      const mathContent = element.getAttribute('data-math');
      if (mathContent && window.katex) {
        try {
          const htmlElement = element as HTMLElement;
          window.katex.render(mathContent, htmlElement, {
            throwOnError: false,
            displayMode: false,
          });
        } catch (error) {
          console.warn('KaTeX render error:', error);
          element.textContent = mathContent;
        }
      }
    });

    // Process block math
    const blockMathElements = contentElement.querySelectorAll('.katex-display');
    blockMathElements.forEach((element: Element) => {
      const mathContent = element.getAttribute('data-math');
      if (mathContent && window.katex) {
        try {
          const htmlElement = element as HTMLElement;
          window.katex.render(mathContent, htmlElement, {
            throwOnError: false,
            displayMode: true,
          });
        } catch (error) {
          console.warn('KaTeX render error:', error);
          element.textContent = mathContent;
        }
      }
    });
  }

  function processCodeBlocks() {
    const codeBlocks = contentElement.querySelectorAll('pre code[class*="language-"]');
    codeBlocks.forEach((block: Element) => {
      if (window.Prism) {
        window.Prism.highlightElement(block);
      }
    });
  }

  function processMermaidDiagrams() {
    const mermaidElements = contentElement.querySelectorAll('.mermaid');
    if (mermaidElements.length > 0 && window.mermaid) {
      window.mermaid.init(undefined, mermaidElements);
    }
  }

  function processHeadings() {
    const headings = contentElement.querySelectorAll('h1, h2, h3, h4, h5, h6');
    headings.forEach((heading: Element) => {
      if (!heading.id) return;
      
      // Add click handler for smooth scrolling
      heading.addEventListener('click', (e) => {
        if (e.target && (e.target as Element).tagName.startsWith('H')) {
          const id = (e.target as Element).id;
          if (id) {
            scrollToHeading(id);
          }
        }
      });
    });
  }

  function scrollToHeading(id: string) {
    const element = contentElement.querySelector(`#${id}`);
    if (element) {
      element.scrollIntoView({ 
        behavior: 'smooth',
        block: 'start',
      });
    }
  }

  function handleScroll(event: Event) {
    if (isScrolling) return;

    isScrolling = true;
    clearTimeout(scrollTimeout);
    
    scrollTimeout = window.setTimeout(() => {
      isScrolling = false;
      
      // Calculate current line based on scroll position
      const target = event.target as HTMLElement;
      const scrollTop = target.scrollTop;
      const lineHeight = 24; // Approximate line height
      const currentLine = Math.floor(scrollTop / lineHeight) + 1;
      
      dispatch('lineScrolled', { line: currentLine });
    }, 100);
  }

  // Global window object extensions for libraries
  declare global {
    interface Window {
      katex?: any;
      mermaid?: any;
      Prism?: any;
    }
  }

  // Load external libraries
  onMount(async () => {
    // Load KaTeX
    if (!window.katex) {
      try {
        const katex = await import('katex');
        window.katex = katex.default;
        
        // Load KaTeX CSS
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = 'https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.css';
        document.head.appendChild(link);
      } catch (error) {
        console.warn('Failed to load KaTeX:', error);
      }
    }

    // Load Prism
    if (!window.Prism) {
      try {
        const Prism = await import('prismjs');
        window.Prism = Prism.default;
        
        // Load additional language support
        await import('prismjs/components/prism-typescript');
        await import('prismjs/components/prism-json');
        await import('prismjs/components/prism-bash');
        await import('prismjs/components/prism-python');
        await import('prismjs/components/prism-rust');
        await import('prismjs/components/prism-sql');
        
        // Load Prism CSS theme
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = 'https://cdn.jsdelivr.net/npm/prismjs@1.29.0/themes/prism.min.css';
        document.head.appendChild(link);
      } catch (error) {
        console.warn('Failed to load Prism:', error);
      }
    }

    // Load Mermaid
    if (!window.mermaid) {
      try {
        const mermaid = await import('mermaid');
        window.mermaid = mermaid.default;
        
        mermaid.default.initialize({
          startOnLoad: false,
          theme: 'default',
          securityLevel: 'loose',
        });
      } catch (error) {
        console.warn('Failed to load Mermaid:', error);
      }
    }

    // Process content after libraries are loaded
    setTimeout(processContent, 100);
  });
</script>

<div class="markdown-view" on:scroll={handleScroll}>
  <article bind:this={contentElement} class="markdown-content">
    {@html document.html}
  </article>
</div>

<style>
  .markdown-view {
    flex: 1;
    overflow-y: auto;
    height: 100%;
    scroll-behavior: smooth;
  }

  .markdown-content {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    line-height: 1.7;
    font-size: 16px;
  }

  /* Markdown content styles */
  :global(.markdown-content h1) {
    font-size: 2.5rem;
    font-weight: 700;
    margin: 2rem 0 1.5rem 0;
    border-bottom: 2px solid var(--color-border);
    padding-bottom: 0.5rem;
    color: var(--color-text-primary);
  }

  :global(.markdown-content h2) {
    font-size: 2rem;
    font-weight: 600;
    margin: 1.75rem 0 1rem 0;
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0.25rem;
    color: var(--color-text-primary);
  }

  :global(.markdown-content h3) {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 1.5rem 0 0.75rem 0;
    color: var(--color-text-primary);
  }

  :global(.markdown-content h4) {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 1.25rem 0 0.5rem 0;
    color: var(--color-text-primary);
  }

  :global(.markdown-content h5),
  :global(.markdown-content h6) {
    font-size: 1rem;
    font-weight: 600;
    margin: 1rem 0 0.5rem 0;
    color: var(--color-text-primary);
  }

  :global(.markdown-content p) {
    margin: 0 0 1.25rem 0;
    color: var(--color-text-primary);
  }

  :global(.markdown-content blockquote) {
    margin: 1.5rem 0;
    padding: 1rem 1.5rem;
    border-left: 4px solid var(--color-accent);
    background-color: var(--color-bg-secondary);
    border-radius: var(--radius);
    font-style: italic;
    color: var(--color-text-secondary);
  }

  :global(.markdown-content code) {
    background-color: var(--color-code-bg);
    color: var(--color-code-text);
    padding: 0.125rem 0.375rem;
    border-radius: var(--radius);
    font-family: var(--font-mono);
    font-size: 0.875em;
    border: 1px solid var(--color-code-border);
  }

  :global(.markdown-content pre) {
    background-color: var(--color-code-bg);
    border: 1px solid var(--color-code-border);
    border-radius: var(--radius);
    padding: 1.5rem;
    margin: 1.5rem 0;
    overflow-x: auto;
    font-family: var(--font-mono);
    font-size: 0.875rem;
    line-height: 1.4;
  }

  :global(.markdown-content pre code) {
    background: none;
    border: none;
    padding: 0;
    font-size: inherit;
  }

  :global(.markdown-content table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1.5rem 0;
    font-size: 0.875rem;
  }

  :global(.markdown-content th),
  :global(.markdown-content td) {
    border: 1px solid var(--color-border);
    padding: 0.75rem 1rem;
    text-align: left;
  }

  :global(.markdown-content th) {
    background-color: var(--color-bg-secondary);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  :global(.markdown-content td) {
    color: var(--color-text-primary);
  }

  :global(.markdown-content ul),
  :global(.markdown-content ol) {
    margin: 1rem 0;
    padding-left: 2rem;
  }

  :global(.markdown-content li) {
    margin: 0.5rem 0;
    color: var(--color-text-primary);
  }

  :global(.markdown-content a) {
    color: var(--color-accent);
    text-decoration: none;
    border-bottom: 1px solid transparent;
    transition: border-color 0.2s ease;
  }

  :global(.markdown-content a:hover) {
    border-bottom-color: var(--color-accent);
  }

  :global(.markdown-content hr) {
    border: none;
    border-top: 2px solid var(--color-border);
    margin: 2rem 0;
  }

  :global(.markdown-content img) {
    max-width: 100%;
    height: auto;
    border-radius: var(--radius);
    box-shadow: var(--shadow-sm);
    margin: 1rem 0;
  }

  /* Task lists */
  :global(.markdown-content input[type="checkbox"]) {
    margin-right: 0.5rem;
    accent-color: var(--color-accent);
  }

  /* KaTeX styles */
  :global(.katex) {
    font-size: 1.1em;
  }

  :global(.katex-display) {
    margin: 1.5rem 0;
    text-align: center;
  }

  /* Mermaid diagrams */
  :global(.mermaid) {
    text-align: center;
    margin: 2rem 0;
  }

  /* Scroll indicators for code blocks */
  :global(.markdown-content pre::-webkit-scrollbar) {
    height: 8px;
  }

  :global(.markdown-content pre::-webkit-scrollbar-track) {
    background: var(--color-bg-tertiary);
    border-radius: 4px;
  }

  :global(.markdown-content pre::-webkit-scrollbar-thumb) {
    background: var(--color-border-hover);
    border-radius: 4px;
  }

  :global(.markdown-content pre::-webkit-scrollbar-thumb:hover) {
    background: var(--color-text-tertiary);
  }
</style>
