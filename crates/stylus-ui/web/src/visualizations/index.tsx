import { JSX } from "react";
import { Visualization } from "../types";

import { TableVisualization } from './TableVisualization.tsx';
import { IframeVisualization } from './IframeVisualization.tsx';
import { SVGVisualization } from './SVGVisualization.tsx';
import { StackVisualization } from './StackVisualization.tsx';
import { IsoflowVisualization } from './IsoflowVisualization.tsx';
import { RowVisualization } from './RowVisualization.tsx';
import { VisualizationState } from "./VisualizationState.tsx";

export function getVisualizationContent(
    visualization: Visualization,
    state: VisualizationState
): JSX.Element {
    const { statusData } = state;
    const hasStatusData = statusData !== null;
    
    switch (visualization.type) {
        case 'table':
            return <TableVisualization state={state} />;
        case 'iframe':
            return (
                <IframeVisualization 
                    state={state}
                    url={visualization.url} 
                    inject={visualization.inject}
                />
            );
        case 'isoflow':
            return <IsoflowVisualization state={state} config={visualization.config} />;
        case 'svg':
            return <SVGVisualization state={state} url={visualization.url} />;
        case 'stack':
            return <StackVisualization state={state} stacks={visualization.stacks} size={visualization.size} />;
        case 'row':
            return <RowVisualization state={state} columns={visualization.columns || []} />;
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
}
