import { JSX } from "react";
import { MonitorStatus, Status } from "./types.ts";

// Helper function for status handling
export function getStatusClass(status: Status): string {
    switch (status) {
        case 'green': return 'status-success';
        case 'yellow': return 'status-timeout';
        case 'red': return 'status-error';
        case 'blank': return 'status-blank';
        default: return 'status-blank';
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
    status: Status | MonitorStatus;
    className?: string;
    title?: string;
}

export function StatusIndicator({ status, className = "", title = "" }: StatusIndicatorProps): JSX.Element {
    if (typeof status === "object") {
        if (status?.metadata) {
            for (const [key, value] of Object.entries(status.metadata)) {
                title += `\n${key}: ${value}`;
            }
        }
        status = status?.status || "blank";
    }
    return (
        <span className={`status-indicator ${getStatusClass(status)} ${className}`} title={title}></span>
    );
} 

export function getStatus(monitor: { status: MonitorStatus }): Status {
    return monitor?.status?.status || "blank";
}
