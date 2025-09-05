import { StatusIndicator } from "../utils.tsx";
import { VisualizationState } from "./VisualizationState.tsx";

interface TableVisualizationProps {
    state: VisualizationState;
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
                        <div key={monitor.id} className={`grid-row status-${monitor.status.status}`} onClick={() => onShowLog(monitor.id, monitor.status.log)}>
                            <div className="grid-cell">
                                <StatusIndicator status={monitor.status.status} />
                                {monitor.id}
                                {Object.keys(monitor.children).length > 0 && (
                                    <span className="children-indicator">
                                        {Object.keys(monitor.children).map((childId) => (
                                            <StatusIndicator key={childId} status={monitor.children[childId].status?.status || "blank"} className="small-status-indicator" />
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
}
