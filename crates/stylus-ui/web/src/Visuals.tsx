import { JSX } from "react";
import { Visualization, StatusData } from "./types";
import { StackVisualization, IframeVisualization, SVGVisualization, IsoflowVisualization, TableVisualization } from "./visualizations/index.ts";

// Base Visualization Card Component
interface VisualizationCardProps {
    visualization: Visualization;
    statusData: StatusData | null;
    onShowLog: (monitorId: string, logEntries: string[]) => void;
    isFullscreen?: boolean;
    onFullscreen?: (visualizationName: string) => void;
}

function VisualizationCard({ visualization, statusData, onShowLog, isFullscreen = false, onFullscreen }: VisualizationCardProps) {
    const getVisualizationContent = () => {
        const hasStatusData = statusData !== null;
        
        switch (visualization.type) {
            case 'table':
                return <TableVisualization statusData={statusData} onShowLog={onShowLog} />;
            case 'iframe':
                return (
                    <IframeVisualization 
                        url={visualization.url} 
                        inject={visualization.inject}
                        statusData={statusData}
                    />
                );
            case 'isoflow':
                return <IsoflowVisualization config={visualization.config} statusData={statusData} />;
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
