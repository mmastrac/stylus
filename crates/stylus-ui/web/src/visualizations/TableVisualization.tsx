import { Monitor, MonitorChildStatus, Status } from "../types.ts";
import { StatusIndicator } from "../utils.tsx";
import { VisualizationState } from "./VisualizationState.tsx";

interface TableVisualizationProps {
    state: VisualizationState;
}

function ChildrenIndicator({ monitor }: { monitor: Monitor }) {
    const childKeys = Object.keys(monitor.children);
    if (childKeys.length <= 8) {
        return Object.keys(monitor.children).length > 0 && (
            <span className="children-indicator">
                {Object.keys(monitor.children).map((childId) => (
                    <StatusIndicator key={childId} status={monitor.children[childId].status?.status || "blank"} className="small-status-indicator" />
                ))}
            </span>
        );
    }

    // If we have more than 8 children, group by state (except red)
    const failed: [string, MonitorChildStatus][] = [];
    const childStatusCounts = Object.keys(monitor.children).reduce((acc, childId) => {
        const child = monitor.children[childId];
        const status = child.status?.status || "blank";
        if (status === 'red') {
            failed.push([childId, child]);
        } else {
            acc[status] = (acc[status] || 0) + 1;
        }
        return acc;
    }, {} as Record<Status, number>);

    return (
        <span className="children-indicator">
            {Object.entries(childStatusCounts).map(([status, count]) => (
                <span>{count}·<StatusIndicator key={status} status={status as Status} className="small-status-indicator" />&nbsp;</span>)
            )}
            {<span>{failed.map(([childId, child]) => {
                return <StatusIndicator key={childId} status={child.status?.status || "blank"} className="small-status-indicator" />
            })}</span>}
        </span>
    );
}

export function TableVisualization({ state }: TableVisualizationProps) {
    const { statusData, onShowLog } = state;
    const hasStatusData = statusData !== null;
    const monitorCount = statusData?.monitors?.length || 0;
    
    return (
        <>
            <div className="visualization-grid-table">
                <div className="grid-header">
                    <div className="grid-cell">Monitor</div>
                    <div className="grid-cell">Status</div>
                </div>
                <div className="grid-body">
                    {statusData?.monitors.map((monitor) => (
                        <div key={monitor.id} className={`grid-row status-${monitor.status.status}`} onClick={() => onShowLog(monitor.id)}>
                            <div className="grid-cell">
                                <StatusIndicator status={monitor.status.status} />
                                {monitor.id}
                                {ChildrenIndicator({ monitor })}
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
}
