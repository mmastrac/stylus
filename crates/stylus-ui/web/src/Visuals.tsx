import { JSX, useEffect, useRef, useCallback } from "react";
import { Visualization, StatusData } from "./types";
import { StatusIndicator } from "./utils.tsx";
import IsoFlow from "isoflow";

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

// SVG Visualization Component
interface SVGVisualizationProps {
    url?: string;
    statusData: StatusData | null;
}

function SVGVisualization({ url, statusData }: SVGVisualizationProps) {
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

// Iframe Visualization Component
interface IframeVisualizationProps {
    url?: string;
    inject?: boolean;
    statusData: StatusData | null;
}

function IframeVisualization({ url, inject, statusData }: IframeVisualizationProps) {
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

// Stack Visualization Component
interface StackRow {
    id: string;
    size: string;
    layout: string;
}

interface Stack {
    title: string;
    rows: StackRow[];
}

interface StackVisualizationProps {
    stacks?: Stack[];
    statusData: StatusData | null;
    size?: 'small' | 'large';
}

function StackVisualization({ stacks, statusData, size }: StackVisualizationProps) {
    if (!stacks || !statusData) {
        return (
            <div className={`visualization-stack ${size ? `stack-${size}` : ''}`}>
                <div className="stack-placeholder">No stack data available</div>
            </div>
        );
    }

    const parseLayout = (layout: string) => {
        // Parse layout like "1x5x2 1x1x2" into multiple group definitions
        const groupDefs = layout.split(' ').map(groupLayout => {
            const parts = groupLayout.split('x').map(Number);
            if (parts.length === 3) {
                return { groups: parts[0], columns: parts[1], rows: parts[2] };
            }
            return { groups: 1, columns: 1, rows: 1 };
        });
        return groupDefs;
    };

    const getChildStatus = (monitorId: string) => {
        const monitor = statusData.monitors.find(m => m.id === monitorId);
        if (!monitor?.children) return [];
        
        return Object.entries(monitor.children)
            .map(([id, child]) => ({ id, child }))
            .sort((a, b) => {
                const aIndex = a.child.axes.index;
                const bIndex = b.child.axes.index;
                if (aIndex !== undefined && bIndex !== undefined) {
                    return aIndex - bIndex;
                }
                return a.id.localeCompare(b.id);
            });
    };

    const renderStackGroups = (children: Array<{id: string, child: any}>, layoutDefs: Array<{groups: number, columns: number, rows: number}>) => {
        let childIndex = 0;
        
        return layoutDefs.map((layoutDef, layoutIndex) => {
            const groups = [];
            
            for (let groupIndex = 0; groupIndex < layoutDef.groups; groupIndex++) {
                const groupChildren = [];
                
                for (let row = 0; row < layoutDef.rows; row++) {
                    for (let col = 0; col < layoutDef.columns; col++) {
                        const child = children[childIndex];
                        childIndex++;
                        
                        if (child) {
                            groupChildren.push(
                                <StatusIndicator 
                                    key={`${row}-${col}`}
                                    status={child.child.status.status} 
                                    className="stack-status-indicator"
                                    title={child.id}
                                />
                            );
                        } else {
                            groupChildren.push(
                                <div key={`${row}-${col}`} className="stack-cell-empty" title={`Empty slot ${childIndex}`} />
                            );
                        }
                    }
                }
                
                groups.push(
                    <div key={groupIndex} className="stack-group" style={{
                        display: 'grid',
                        gridTemplateColumns: `repeat(${layoutDef.columns}, var(--grid-size, 8px))`,
                        gridTemplateRows: `repeat(${layoutDef.rows}, var(--grid-size, 8px))`,
                        gap: 'var(--grid-gap, 1px)'
                    }}>
                        {groupChildren}
                    </div>
                );
            }
            
            return groups;
        });
    };

    return (
        <div className={`visualization-stack ${size ? `stack-${size}` : ''}`}>
            <div className="stack-columns">
                {stacks.map((stack, stackIndex) => (
                    <div key={stackIndex} className="stack-column">
                        <h4 className="stack-title">{stack.title}</h4>
                        {stack.rows.map((row, rowIndex) => {
                            const children = getChildStatus(row.id);
                            const layoutDefs = parseLayout(row.layout);
                            
                            return (
                                <div key={rowIndex} className="stack-row">
                                    <div className="stack-row-header">
                                        <span className="stack-row-id">{row.id}</span>
                                    </div>
                                    <div className="stack-groups">
                                        {renderStackGroups(children, layoutDefs)}
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                ))}
            </div>
        </div>
    );
}

// Base Visualization Card Component
interface VisualizationCardProps {
    visualization: Visualization;
    statusData: StatusData | null;
    onShowLog: (monitorId: string, logEntries: string[]) => void;
    isFullscreen?: boolean;
    onFullscreen?: (visualizationName: string) => void;
}

function updateIsoflowDataWithStatus(data: any, statusData: StatusData | null) {
    if (!statusData || !data) {
        return data;
    }

    // Deep clone
    let initialData = JSON.parse(JSON.stringify(data));

    const items = new Map<string, any>(initialData.items.map(
        (item: {id: string, name: string}) => [item.id, item]));

    // Always use our own colors
    const colors = initialData.colors;
    colors.push({ id: "stylus-blank", value: "#555555" });
    colors.push({ id: "stylus-red", value: "#ff5555" });
    colors.push({ id: "stylus-green", value: "#55cc55" });
    colors.push({ id: "stylus-yellow", value: "#ffee11" });

    const view = initialData.views[0];
    const viewItems = view.items;
    const connectors = view.connectors;
    const rectangles = view.rectangles || (view.rectangles = []);

    // For each of our status items, create a rectangle
    // for the item with a matching id.
    for (const item of viewItems) {
        let monitor;
        for (const monitorItem of statusData.monitors) {
            const itemData = items.get(item.id);
            if (itemData?.name.startsWith(monitorItem.id) || item.id == monitorItem.id) {
                monitor = monitorItem;
                itemData.description = (itemData.description || "") + "<p><b>" + monitor.status.description + "</b></p>";
                break;
            }
        }

        const COLOR_MAP = {
            'red': 'stylus-red',
            'yellow': 'stylus-yellow',
            'green': 'stylus-green',
            'blank': 'stylus-blank',
        }

        if (monitor && COLOR_MAP[monitor.status.status] !== undefined) {
            const color = COLOR_MAP[monitor.status.status];
            const rectangle = {
                "id": `rect-status-${item.id}`,
                "color": color,
                "from": item.tile,
                "to": item.tile
            };
            rectangles.unshift(rectangle);
        }
    }

    for (const connector of connectors) {
        let monitor;
        for (const monitorItem of statusData.monitors) {
            if (connector.id.startsWith(monitorItem.id)) {
                monitor = monitorItem;
                break;
            }
        }

        if (monitor && monitor.status.metadata?.rps) {
            connector.description = monitor.status.metadata.rps;
        }
    }

    return initialData;
}

function VisualizationCard({ visualization, statusData, onShowLog, isFullscreen = false, onFullscreen }: VisualizationCardProps) {
    const getVisualizationContent = () => {
        const hasStatusData = statusData !== null;
        const monitorCount = statusData?.monitors?.length || 0;
        
        switch (visualization.type) {
            case 'table':
                return (
                    <>
                        <div className="visualization-grid-table">
                            <div className="grid-header">
                                <div className="grid-cell">Monitor</div>
                                <div className="grid-cell">Status</div>
                            </div>
                            <div className="grid-body">
                                {statusData?.monitors.map((monitor) => (
                                    <div key={monitor.id} className={`grid-row status-${monitor.status.status}`} onClick={() => onShowLog(monitor.id, monitor.status.log)}>
                                        <div className="grid-cell">
                                            <StatusIndicator status={monitor.status.status} />
                                            {monitor.id}
                                            {Object.keys(monitor.children).length > 0 && (
                                                <span className="children-indicator">
                                                    {Object.keys(monitor.children).map((childId) => (
                                                        <StatusIndicator key={childId} status={monitor.children[childId].status.status} className="small-status-indicator" />
                                                    ))}
                                                </span>
                                            )}
                                        </div>
                                        <div className="grid-cell">{monitor.status.description} ({monitor.status.code})</div>
                                    </div>
                                ))}
                            </div>
                        </div>
                        <div className="visualization-info">
                            Status data: {hasStatusData ? 'Available' : 'Not available'} 
                            {hasStatusData && ` (${monitorCount} monitors)`}
                        </div>
                    </>
                );
            case 'iframe':
                return (
                    <IframeVisualization 
                        url={visualization.url} 
                        inject={visualization.inject}
                        statusData={statusData}
                    />
                );
            case 'isoflow':
                const initialData = updateIsoflowDataWithStatus(statusData?.config.config_d[visualization.config || "isoflow"], statusData);
                return (
                    <div className="visualization-isoflow" style={{ width: '100%', height: '100%' }}>
                        <div className="isoflow-placeholder" style={{ width: '100%', height: '100%' }}>
                            <IsoFlow width="100%" height="100%" enableGlobalDragHandlers={false} initialData={initialData} editorMode="EXPLORABLE_READONLY" />
                        </div>
                    </div>
                );
            case 'svg':
                return <SVGVisualization url={visualization.url} statusData={statusData} />;
            case 'stack':
                return <StackVisualization stacks={visualization.stacks} statusData={statusData} size={visualization.size} />;
            default:
                return (
                    <>
                        <p>Unknown visualization type: {visualization.type}</p>
                        <p className="visualization-info">
                            Status data: {hasStatusData ? 'Available' : 'Not available'}
                        </p>
                    </>
                );
        }
    };

    const getVisualizationClass = () => {
        if (isFullscreen) return 'fullscreen';
        
        // Determine if visualization is greedy or sized
        switch (visualization.type) {
            case 'table':
            case 'svg':
            case 'stack':
                return 'sized';
            case 'iframe':
            case 'isoflow':
                return 'greedy';
            default:
                return 'sized';
        }
    };

    return (
        <div className={`visualization-card ${getVisualizationClass()}`}>
            <div className="visualization-card-content">
                {!isFullscreen && <h3>{visualization.title}</h3>}
                {!isFullscreen && <p>{visualization.description}</p>}
                {getVisualizationContent()}
            </div>
            {!isFullscreen && onFullscreen && (
                <button 
                    className="fullscreen-button"
                    onClick={() => onFullscreen(visualization.title)}
                    title="Open in fullscreen"
                >
                    ⛶
                </button>
            )}
        </div>
    );
}

export { VisualizationCard };

// Visualization Grid Component
interface VisualizationGridProps {
    visualizations: Visualization[];
    statusData: StatusData | null;
    onShowLog: (monitorId: string, logEntries: string[]) => void;
    onFullscreen?: (visualizationName: string) => void;
}

export function VisualizationGrid({ visualizations, statusData, onShowLog, onFullscreen }: VisualizationGridProps): JSX.Element {
    if (!visualizations || visualizations.length === 0) {
        return (
            <div className="content">
                <div className="visualization-placeholder">
                    <h2>No Visualizations</h2>
                    <p>No visualizations are currently configured.</p>
                </div>
            </div>
        );
    }

    return (
        <div className="content">
            <div className="visualization-grid">
                {visualizations.map((visualization, index) => (
                    <VisualizationCard 
                        key={`visualization-${index}`}
                        visualization={visualization}
                        statusData={statusData}
                        onShowLog={onShowLog}
                        onFullscreen={onFullscreen}
                    />
                ))}
            </div>
        </div>
    );
}
