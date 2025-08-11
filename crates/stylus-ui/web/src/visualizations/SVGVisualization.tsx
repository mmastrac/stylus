import { useEffect, useRef } from "react";
import { VisualizationState } from "./VisualizationState.tsx";

// SVG Visualization Component
interface SVGVisualizationProps {
    state: VisualizationState;
    url?: string;
}

export function SVGVisualization({ state, url }: SVGVisualizationProps) {
    const { statusData } = state;
    const containerRef = useRef<HTMLDivElement>(null);
    const shadowRootRef = useRef<ShadowRoot | null>(null);
    const styleRef = useRef<HTMLStyleElement | null>(null);

    useEffect(() => {
        if (!url || !containerRef.current) return;

        // Create shadow root if it doesn't exist
        if (!shadowRootRef.current) {
            shadowRootRef.current = containerRef.current.attachShadow({ mode: 'open' });
        }

        const shadowRoot = shadowRootRef.current;

        // Create or update style element
        if (!styleRef.current) {
            styleRef.current = document.createElement('style');
            shadowRoot.appendChild(styleRef.current);
        }

        // Fetch CSS directly with cache buster
        fetch(`/style.css?t=${new Date().valueOf()}`)
            .then(res => res.text())
            .then(cssText => {
                styleRef.current!.textContent = cssText;
            })
            .catch(error => {
                console.error('Failed to load CSS for SVG visualization:', error);
            });

        // Find or create SVG container
        let svgContainer = shadowRoot.querySelector('.svg-visualization-container') as HTMLElement;
        if (!svgContainer) {
            svgContainer = document.createElement('div');
            svgContainer.className = 'svg-visualization-container';
            svgContainer.style.width = '100%';
            svgContainer.style.height = '100%';
            svgContainer.style.overflow = 'hidden';
            shadowRoot.appendChild(svgContainer);
        }

        // Load SVG with cache buster to force re-render when status changes
        const loadSVG = fetch(`${url}?t=${new Date().valueOf()}`)
            .then(res => res.text())
            .then(svgText => {
                // Update SVG content
                svgContainer.innerHTML = svgText;
                
                // Make SVG use natural size but bound by container
                const svg = svgContainer.querySelector('svg');
                if (svg) {
                    svg.setAttribute('class', 'svg-visualization-element');
                    svg.style.maxWidth = '100%';
                    svg.style.maxHeight = '100%';
                    svg.style.width = 'auto';
                    svg.style.height = 'auto';
                    svg.style.display = 'block';
                }
            });

        loadSVG
            .catch(error => {
                console.error('Failed to load SVG visualization:', error);
                const errorDiv = document.createElement('div');
                errorDiv.className = 'svg-visualization-error';
                errorDiv.style.padding = '20px';
                errorDiv.style.color = 'red';
                errorDiv.innerHTML = `
                    <h3>Failed to load SVG</h3>
                    <p>Error: ${error.message}</p>
                    <p>URL: ${url}</p>
                `;
                shadowRoot.appendChild(errorDiv);
            });
    }, [url, statusData]);

    return (
        <div 
            ref={containerRef}
            className="visualization-svg svg-visualization-root"
            style={{ width: '100%', height: '100%' }}
        />
    );
}
