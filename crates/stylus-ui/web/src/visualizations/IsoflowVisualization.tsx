import IsoFlow from "isoflow";
import { VisualizationState } from "./VisualizationState.tsx";

function updateIsoflowDataWithStatus(data: any, statusData: any) {
    if (!statusData || !data) {
        return data;
    }

    // Deep clone
    let initialData = JSON.parse(JSON.stringify(data));

    const items = new Map<string, any>(initialData.items.map(
        (item: {id: string, name: string}) => [item.id, item]));

    // Always use our own colors
    const colors = initialData.colors;
    colors.push({ id: "stylus-blank", value: "#555555" });
    colors.push({ id: "stylus-red", value: "#ff5555" });
    colors.push({ id: "stylus-green", value: "#55cc55" });
    colors.push({ id: "stylus-yellow", value: "#ffee11" });

    const view = initialData.views[0];
    const viewItems = view.items;
    const connectors = view.connectors;
    const rectangles = view.rectangles || (view.rectangles = []);

    // For each of our status items, create a rectangle
    // for the item with a matching id.
    for (const item of viewItems) {
        let monitor;
        for (const monitorItem of statusData.monitors) {
            const itemData = items.get(item.id);
            if (itemData?.name.startsWith(monitorItem.id) || item.id == monitorItem.id) {
                monitor = monitorItem;
                itemData.description = (itemData.description || "") + "<p><b>" + monitor.status.description + "</b></p>";
                break;
            }
        }

        const COLOR_MAP = {
            'red': 'stylus-red',
            'yellow': 'stylus-yellow',
            'green': 'stylus-green',
            'blank': 'stylus-blank',
        }

        if (monitor && COLOR_MAP[monitor.status.status as keyof typeof COLOR_MAP] !== undefined) {
            const color = COLOR_MAP[monitor.status.status as keyof typeof COLOR_MAP];
            const rectangle = {
                "id": `rect-status-${item.id}`,
                "color": color,
                "from": item.tile,
                "to": item.tile
            };
            rectangles.unshift(rectangle);
        }
    }

    for (const connector of connectors) {
        let monitor;
        for (const monitorItem of statusData.monitors) {
            if (connector.id.startsWith(monitorItem.id)) {
                monitor = monitorItem;
                break;
            }
        }

        if (monitor && monitor.status.metadata?.rps) {
            connector.description = monitor.status.metadata.rps;
        }
    }

    return initialData;
}

interface IsoflowVisualizationProps {
    config?: string;
    state: VisualizationState;
}

export function IsoflowVisualization({ state, config }: IsoflowVisualizationProps) {
    const { statusData } = state;
    const initialData = updateIsoflowDataWithStatus(statusData?.config.config_d[config || "isoflow"], statusData);
    
    return (
        <div className="visualization-isoflow" style={{ width: '100%', height: '100%' }}>
            <div className="isoflow-placeholder" style={{ width: '100%', height: '100%' }}>
                <IsoFlow width="100%" height="100%" enableGlobalDragHandlers={false} initialData={initialData} editorMode="EXPLORABLE_READONLY" />
            </div>
        </div>
    );
}
