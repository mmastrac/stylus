import { JSX } from "react";
import { Visualization, StatusData, RowColumn } from "./types";
import { getVisualizationContent } from "./visualizations/index.tsx";

// Base Visualization Card Component
interface VisualizationCardProps {
    visualization: Visualization;
    statusData: StatusData | null;
    onShowLog: (monitorId: string) => void;
    isFullscreen?: boolean;
    onFullscreen?: (visualizationName: string) => void;
}

const getVisualizationClass = (isFullscreen: boolean, visualization: { type: string, columns?: RowColumn[] }) => {
    if (isFullscreen) return 'fullscreen';
    
    // Determine if visualization is greedy or sized
    switch (visualization.type) {
        case 'table':
        case 'svg':
        case 'stack':
        case 'row':
            for (const column of visualization.columns || []) {
                if (getVisualizationClass(isFullscreen, column) === 'greedy') {
                    return 'greedy';
                }
            };
            return 'sized';
        case 'iframe':
        case 'isoflow':
            return 'greedy';
        default:
            return 'sized';
    }
};

export function VisualizationCard({ visualization, statusData, onShowLog, isFullscreen = false, onFullscreen }: VisualizationCardProps) {
    const visualizationState = {
        statusData,
        onShowLog,
        onFullscreen,
        getVisualizationContent
    };

    return (
        <div className={`visualization-card ${getVisualizationClass(isFullscreen, visualization)}`}>
            <div className="visualization-card-content">
                {!isFullscreen && <h3>{visualization.title}</h3>}
                {!isFullscreen && <p>{visualization.description}</p>}
                {getVisualizationContent(visualization, visualizationState)}
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

// Visualization Grid Component
interface VisualizationGridProps {
    visualizations: Visualization[];
    statusData: StatusData | null;
    onShowLog: (monitorId: string) => void;
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
