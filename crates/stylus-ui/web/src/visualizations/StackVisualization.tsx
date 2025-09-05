
import { StatusIndicator } from "../utils.tsx";
import { VisualizationState } from "./VisualizationState.tsx";

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
    size?: 'small' | 'large';
    state: VisualizationState;
}

export function StackVisualization({ state, stacks, size }: StackVisualizationProps) {
    const { statusData } = state;
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
            if (parts.length === 2) {
                return { groups: 1, columns: parts[0], rows: parts[1] };
            }
            if (parts.length === 1) {
                return { groups: 1, columns: parts[0], rows: 1 };
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
        
        return layoutDefs.map((layoutDef) => {
            const groups = [];
            
            for (let groupIndex = 0; groupIndex < layoutDef.groups; groupIndex++) {
                const groupChildren = [];
                
                for (let row = 0; row < layoutDef.rows; row++) {
                    for (let col = 0; col < layoutDef.columns; col++) {
                        const child = children[childIndex];
                        childIndex++;
                        
                        if (child?.child?.status?.metadata) {
                            let title = `${child.id}`;
                            for (const [key, value] of Object.entries(child.child.status.metadata)) {
                                title += `\n${key}: ${value}`;
                            }
                            groupChildren.push(
                                <StatusIndicator 
                                    key={`${row}-${col}`}
                                    status={child.child.status.status} 
                                    className="stack-status-indicator"
                                    title={title}
                                />
                            );
                        } else {
                            groupChildren.push(
                                <StatusIndicator 
                                    key={`${row}-${col}`}
                                    status="blank"
                                    className="stack-status-indicator"
                                    title={`No child found for port index ${childIndex}`}
                                />
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
