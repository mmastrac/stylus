import { useEffect, useRef, useCallback } from "react";
import { VisualizationState } from "./VisualizationState.tsx";

// Common function to inject styles into a document
function injectStylesIntoDocument(document: Document, cacheBuster: boolean = false): void {
    const styleId = '__stylus_style__';
    const oldLink = document.getElementById(styleId) as HTMLLinkElement;
    if (oldLink) {
        oldLink.id = "";
    }
    
    const css = document.createElement('link');
    css.rel = "stylesheet";
    css.href = cacheBuster ? `/style.css?t=${new Date().valueOf()}` : '/style.css';
    css.id = styleId;
    
    css.onload = function() {
        if (oldLink) {
            oldLink.remove();
        }
        css.id = styleId;
    };
    
    css.onerror = function() {
        if (oldLink) {
            oldLink.remove();
        }
        css.remove();
        console.error('Failed to load CSS for document');
    };
    
    // Insert as first child of head
    if (document.head.firstChild) {
        document.head.insertBefore(css, document.head.firstChild);
    } else {
        document.head.appendChild(css);
    }
}

// Iframe Visualization Component
interface IframeVisualizationProps {
    state: VisualizationState;
    url?: string;
    inject?: boolean;
}

export function IframeVisualization({ state, url, inject }: IframeVisualizationProps) {
    const { statusData } = state;
    const iframeRef = useRef<HTMLIFrameElement>(null);
    const isLoadedRef = useRef<boolean>(false);

    // Handle style injection when iframe loads or statusData changes
    useEffect(() => {
        if (!inject || !isLoadedRef.current || !iframeRef.current) return;

        const iframeDocument = iframeRef.current?.contentDocument;
        if (iframeDocument) {
            injectStylesIntoDocument(iframeDocument, true);
        }
    }, [inject, statusData, isLoadedRef.current]);

    const setIframeRef = useCallback((element: HTMLIFrameElement | null) => {
        iframeRef.current = element;
        
        // If we have a URL and the iframe element, set it up immediately
        if (element && url) {
            element.src = url;
            
            const handleLoad = () => {
                isLoadedRef.current = true;
                const iframeDocument = iframeRef.current?.contentDocument;
                if (iframeDocument) {
                    injectStylesIntoDocument(iframeDocument, true);
                }
            };
            
            element.addEventListener('load', handleLoad);
            element.addEventListener('DOMContentLoaded', handleLoad);
        }
    }, [url]);

    return (
        <div className="visualization-iframe">
            {url ? (
                <iframe 
                    ref={setIframeRef}
                    className="visualization-url" 
                />
            ) : (
                <div className="visualization-iframe-placeholder">No URL</div>
            )}
        </div>
    );
}
