import { VisualizationState } from "./VisualizationState.tsx";

interface RowColumn {
    type: string;
    width?: number;
    url?: string;
    config?: string;
    inject?: boolean;
    stacks?: any[];
    size?: 'small' | 'large';
}

interface RowVisualizationProps {
    state: VisualizationState;
    columns: RowColumn[];
}

export function RowVisualization({ state, columns }: RowVisualizationProps) {
    const { getVisualizationContent } = state;
    if (!columns || columns.length === 0) {
        return (
            <div className="visualization-row">
                <div className="row-placeholder">No columns configured</div>
            </div>
        );
    }

    // Create grid template columns string using fr units
    // Ensure the total adds up to a reasonable number for proper distribution
    const totalWidth = columns.reduce((sum, col) => sum + (col.width || 1), 0);
    const gridTemplateColumns = columns.map(col => `${col.width || 1}fr`).join(' ');

    // Create visualization objects for each column
    const columnVisualizations = columns.map((column, index) => ({
        title: `Column ${index + 1}`,
        description: `Column ${index + 1}`,
        type: column.type,
        url: column.url,
        config: column.config,
        inject: column.inject,
        stacks: column.stacks,
        size: column.size,
    }));

    return (
        <div className="visualization-row">
            <div 
                className="row-columns" 
                style={{ 
                    display: 'grid', 
                    gridTemplateColumns,
                    gap: '20px',
                    height: '100%',
                    minHeight: '0', // Allow grid to shrink
                    width: '100%'
                }}
            >
                {columnVisualizations.map((visualization, index) => (
                    <div key={index} className="row-column">
                        <div className="visualization-card sized">
                            <div className="visualization-card-content">
                                {getVisualizationContent(visualization, state)}
                            </div>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}


