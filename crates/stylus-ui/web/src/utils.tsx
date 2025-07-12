import { JSX } from "react";
import { Status } from "./types.ts";

// Helper function for status handling
export function getStatusClass(status: Status): string {
    switch (status) {
        case 'green': return 'status-success';
        case 'yellow': return 'status-timeout';
        case 'red': return 'status-error';
        case 'blank': return 'status-blank';
        default: return 'status-error';
    }
}

// URL-safe ID generation and lookup utilities
export function createUrlSafeId(title: string): string {
    return title
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '');
}

export function findVisualizationById(visualizations: any[], urlId: string): any | null {
    return visualizations.find(viz => createUrlSafeId(viz.title) === urlId) || null;
}

// Status Indicator Component
interface StatusIndicatorProps {
    status: Status;
    className?: string;
}

export function StatusIndicator({ status, className = "" }: StatusIndicatorProps): JSX.Element {
    return (
        <span className={`status-indicator ${getStatusClass(status)} ${className}`}></span>
    );
} 