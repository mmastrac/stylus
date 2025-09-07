
import { StatusIndicator } from "../utils.tsx";
import { VisualizationState } from "./VisualizationState.tsx";

// Stack Visualization Component
interface StackRow {
    id: string;
    size: string;
    layout: string;
    order?: string;
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

    // Parse order like "15-22 12 3-4" into an array of numbers [15, 16, ..., 22, 12, 3, 4]
    // Note that reverse order is supported, so "22-15 12 4-3" is valid and will generate the ranges
    // reversed.
    const parseOrder = (order: string) => {
        const result: number[] = [];
        const parts = order.trim().split(/\s+/);
        
        for (const part of parts) {
            if (part.includes('-')) {
                const [startStr, endStr] = part.split('-');
                const start = parseInt(startStr, 10);
                const end = parseInt(endStr, 10);
                
                if (isNaN(start) || isNaN(end)) {
                    continue;
                }
                
                if (start <= end) {
                    // Normal range: 15-22 -> [15, 16, 17, 18, 19, 20, 21, 22]
                    for (let i = start; i <= end; i++) {
                        result.push(i);
                    }
                } else {
                    // Reverse range: 22-15 -> [22, 21, 20, 19, 18, 17, 16, 15]
                    for (let i = start; i >= end; i--) {
                        result.push(i);
                    }
                }
            } else {
                // Single number: 12 -> [12]
                const num = parseInt(part, 10);
                if (!isNaN(num)) {
                    result.push(num);
                }
            }
        }
        
        return result;
    };

    const reorderChildren = (children: Array<{id: string, child: any}>, order: number[]) => {
        // Create a map of children by their index (1-based to match monitor indices)
        const childrenByIndex = new Map<number, {id: string, child: any}>();
        children.forEach((child, index) => {
            // Try to get index from child.child.axes.index first, then fallback to position
            const childIndex = child.child?.axes?.index ?? (index + 1);
            childrenByIndex.set(childIndex, child);
        });
        
        // Reorder according to the specified order
        const reordered: Array<{id: string, child: any}> = [];
        for (const index of order) {
            const child = childrenByIndex.get(index);
            if (child) {
                reordered.push(child);
            }
        }
        
        return reordered;
    };

    const parseLayout = (layout: string) => {
        // Parse layout like "1x5x2 1x1x2" into multiple group definitions
        const groupDefs = layout.split(' ').map(groupLayout => {
            let isColumnWise = false;
            if (groupLayout.startsWith('~')) {
                isColumnWise = true;
                groupLayout = groupLayout.slice(1);
            }
            const parts = groupLayout.split('x').map(Number);
            if (parts.length === 3) {
                return { groups: parts[0], columns: parts[1], rows: parts[2], isColumnWise };
            }
            if (parts.length === 2) {
                return { groups: 1, columns: parts[0], rows: parts[1], isColumnWise };
            }
            if (parts.length === 1) {
                return { groups: 1, columns: parts[0], rows: 1, isColumnWise };
            }
            return { groups: 1, columns: 1, rows: 1, isColumnWise };
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

    const renderStackGroups = (children: Array<{id: string, child: any}>, layoutDefs: Array<{groups: number, columns: number, rows: number, isColumnWise: boolean}>) => {
        let childIndex = 0;
        
        return layoutDefs.map((layoutDef) => {
            const groups = [];
            
            for (let groupIndex = 0; groupIndex < layoutDef.groups; groupIndex++) {
                const groupChildren = [];
                
                let groupStart = childIndex;
                childIndex += layoutDef.columns * layoutDef.rows;
                for (let row = 0; row < layoutDef.rows; row++) {
                    for (let col = 0; col < layoutDef.columns; col++) {
                        const child = layoutDef.isColumnWise 
                            ? children[groupStart + col * layoutDef.rows + row]
                            : children[groupStart + row * layoutDef.columns + col];
                        
                        if (child?.child) {
                            groupChildren.push(
                                <StatusIndicator 
                                    key={`${row}-${col}`}
                                    status={child.child.status} 
                                    className="stack-status-indicator"
                                    title={`${child.id}`}
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
                            
                            // Apply ordering if specified
                            const orderedChildren = row.order ? 
                                reorderChildren(children, parseOrder(row.order)) : 
                                children;
                            
                            return (
                                <div key={rowIndex} className="stack-row">
                                    <div className="stack-row-header">
                                        <span className="stack-row-id">{row.id}</span>
                                    </div>
                                    <div className="stack-groups">
                                        {renderStackGroups(orderedChildren, layoutDefs)}
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
