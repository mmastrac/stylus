import { StatusData, Visualization } from "../types";
import { JSX } from "react";

type GetVisualizationContent = (visualization: Visualization, state: VisualizationState) => JSX.Element;

export interface VisualizationState {
    statusData: StatusData | null;
    onShowLog: (monitorId: string) => void;
    onFullscreen?: (visualizationName: string) => void;
    getVisualizationContent: GetVisualizationContent;
}

