import { JSX, useState, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { StatusData, Monitor, Status } from "./types.ts";
import { VisualizationGrid, VisualizationCard } from "./Visuals.tsx";
import { LogModal } from "./LogViewer.tsx";
import { StatusIndicator, createUrlSafeId, findVisualizationById } from "./utils.tsx";

// Custom hook for fetching status data
function useStatusData() {
    const [data, setData] = useState<StatusData | null>(null);
    const [loading, setLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);

    const fetchData = async (): Promise<void> => {
        try {
            setLoading(true);
            setError(null);
            const response = await fetch('/status.json');
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const result: StatusData = await response.json();
            if (result.config.ui == undefined) {
                setError("Missing UI configuration");
            } else {
                setData(result);
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchData();
        // Set up auto-refresh every 5 seconds
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    return { data, loading, error, refetch: fetchData };
}

// Child Status Card Component (for group monitors)
interface ChildStatusCardProps {
    childId: string;
    childStatus: Status;
}

function ChildStatusCard({ childId, childStatus }: ChildStatusCardProps) {
    return (
        <div 
            title={childId}
        >
            <StatusIndicator status={childStatus} className="child-status-indicator" />
        </div>
    );
}

// Status Card Component
interface StatusCardProps {
    key: string;
    monitor: Monitor;
    onShowLog: (monitorId: string, logEntries: string[]) => void;
}

function StatusCard({ monitor, onShowLog }: StatusCardProps) {

    const isGroupMonitor = monitor.children && Object.keys(monitor.children).length > 0;

    return (
        <div className="status-card">
            <h3>{monitor.id}</h3>
            <div>
                <StatusIndicator status={monitor.status.status} />
                <span>{monitor.status.description}</span>
                <span className="status-code" data-code={monitor.status.code}>{monitor.status.code}</span>
            </div>
            
            {isGroupMonitor && (
                <div className="children-container">
                    <strong>Children:</strong>
                    <div className="children-grid">
                        {Object.entries(monitor.children).map(([childId, childData]) => (
                            <ChildStatusCard 
                                key={childId}
                                childId={childId}
                                childStatus={childData.status.status}
                            />
                        ))}
                    </div>
                </div>
            )}
            
            <div className="status-card-footer">
                {monitor.status.log && monitor.status.log.length > 0 && (
                    <button 
                        className="view-log-button"
                        onClick={() => onShowLog(monitor.id, monitor.status.log)}
                    >
                        View Log <span className="show-popup"></span>
                    </button>
                )}
            </div>
        </div>
    );
}

// Theme toggle hook
function useThemeToggle() {
    type ThemeMode = 'light' | 'dark' | 'auto';
    
    const [themeMode, setThemeMode] = useState<ThemeMode>(() => {
        // Check if user has manually set a preference
        const saved = localStorage.getItem('theme-preference');
        if (saved === 'light' || saved === 'dark' || saved === 'auto') {
            return saved;
        }
        // Default to auto
        return 'auto';
    });

    const [isDark, setIsDark] = useState<boolean>(() => {
        if (themeMode === 'auto') {
            return window.matchMedia('(prefers-color-scheme: dark)').matches;
        }
        return themeMode === 'dark';
    });

    useEffect(() => {
        const root = document.documentElement;
        
        if (themeMode === 'auto') {
            root.classList.remove('light-mode', 'dark-mode');
            localStorage.setItem('theme-preference', 'auto');
            // Listen for system preference changes
            const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
            const handleChange = (e: MediaQueryListEvent) => setIsDark(e.matches);
            mediaQuery.addEventListener('change', handleChange);
            return () => mediaQuery.removeEventListener('change', handleChange);
        } else {
            root.classList.remove('light-mode', 'dark-mode');
            if (themeMode === 'dark') {
                root.classList.add('dark-mode');
            } else {
                root.classList.add('light-mode');
            }
            localStorage.setItem('theme-preference', themeMode);
            setIsDark(themeMode === 'dark');
        }
    }, [themeMode]);

    const cycleTheme = () => {
        const modes: ThemeMode[] = ['light', 'dark', 'auto'];
        const currentIndex = modes.indexOf(themeMode);
        const nextIndex = (currentIndex + 1) % modes.length;
        setThemeMode(modes[nextIndex]);
    };

    return { themeMode, isDark, cycleTheme };
}

// Theme Toggle Button Component
function ThemeToggle({ themeMode, onToggle }: { themeMode: 'light' | 'dark' | 'auto'; onToggle: () => void }) {
    const getIcon = () => {
        switch (themeMode) {
            case 'light': return '☀️';
            case 'dark': return '🌙';
            case 'auto': return '🌗';
        }
    };

    const getTitle = () => {
        switch (themeMode) {
            case 'light': return 'Switch to dark mode';
            case 'dark': return 'Switch to auto mode';
            case 'auto': return 'Switch to light mode';
        }
    };

    return (
        <button 
            className="theme-toggle" 
            onClick={onToggle}
            title={getTitle()}
        >
            {getIcon()}
        </button>
    );
}

// Header Component
interface HeaderProps {
    title: string;
    description: string;
    themeMode: 'light' | 'dark' | 'auto';
    onThemeToggle: () => void;
}

function Header({ title, description, themeMode, onThemeToggle }: HeaderProps) {
    return (
        <div className="header">
            <div className="header-content">
                <div>
                    <h1>{title}</h1>
                    <p>{description}</p>
                </div>
                <ThemeToggle themeMode={themeMode} onToggle={onThemeToggle} />
            </div>
        </div>
    );
}

// Tab Navigation Component
interface TabNavigationProps {
    activeTab: string;
    onTabChange: (tab: string) => void;
}

function TabNavigation({ activeTab, onTabChange }: TabNavigationProps) {
    return (
        <div className="tab-navigation">
            <button 
                className={`tab-button ${activeTab === 'visualization' ? 'active' : ''}`}
                onClick={() => onTabChange('visualization')}
            >
                Visualization
            </button>
            <button 
                className={`tab-button ${activeTab === 'monitors' ? 'active' : ''}`}
                onClick={() => onTabChange('monitors')}
            >
                Monitors
            </button>
        </div>
    );
}

// Fullscreen Visualization Component
interface FullscreenVisualizationProps {
    data: StatusData | null;
    visualizationName: string;
    onShowLog: (monitorId: string, logEntries: string[]) => void;
    onClose: () => void;
}

function FullscreenVisualization({ data, visualizationName, onShowLog, onClose }: FullscreenVisualizationProps) {
    const visualization = data?.config.ui.visualizations ? 
        findVisualizationById(data.config.ui.visualizations, visualizationName) : null;
    
    if (!visualization) {
        return (
            <div className="fullscreen-overlay">
                <div className="fullscreen-content">
                    <div className="fullscreen-header">
                        <h2>Visualization Not Found</h2>
                        <button className="fullscreen-close" onClick={onClose}>×</button>
                    </div>
                    <div className="fullscreen-body">
                        <p>Visualization not found.</p>
                        <p>URL ID: "{visualizationName}"</p>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="fullscreen-overlay">
            <div className="fullscreen-content">
                <div className="fullscreen-header">
                    <h2>{visualization.title}</h2>
                    <button className="fullscreen-close" onClick={onClose}>×</button>
                </div>
                <div className="fullscreen-body">
                    <VisualizationCard 
                        visualization={visualization}
                        statusData={data}
                        onShowLog={onShowLog}
                        isFullscreen={true}
                        key={visualizationName}
                    />
                </div>
            </div>
        </div>
    );
}

// Visualization Tab Component
interface VisualizationTabProps {
    data: StatusData | null;
    onShowLog: (monitorId: string, logEntries: string[]) => void;
    onFullscreen: (visualizationName: string) => void;
}

function VisualizationTab({ data, onShowLog, onFullscreen }: VisualizationTabProps) {
    const visualizations = data?.config.ui.visualizations || [];
    
    return <VisualizationGrid 
        visualizations={visualizations} 
        statusData={data} 
        onShowLog={onShowLog}
        onFullscreen={onFullscreen}
    />;
}

// Monitors Tab Component
interface MonitorsTabProps {
    data: StatusData | null;
    loading: boolean;
    onRefetch: () => void;
    onShowLog: (monitorId: string, logEntries: string[]) => void;
}

function MonitorsTab({ data, loading, onRefetch, onShowLog }: MonitorsTabProps) {
    if (loading && !data) {
        return (
            <div className="content">
                <div className="loading">
                    <h2>Loading monitor status...</h2>
                </div>
            </div>
        );
    }

    return (
        <div className="content">
            {data && data.monitors && data.monitors.length > 0 ? (
                <div className="status-grid">
                    {data.monitors.map((monitor: Monitor) => (
                        <StatusCard 
                            key={monitor.id} 
                            monitor={monitor} 
                            onShowLog={onShowLog}
                        />
                    ))}
                </div>
            ) : (
                <div className="loading">
                    <h2>No monitors found</h2>
                    <p>No monitors are currently configured.</p>
                </div>
            )}
            
            <div style={{ marginTop: '40px', textAlign: 'center' }}>
                <p>
                    <a href="/status.json" target="_blank" rel="noopener noreferrer">
                        View Raw JSON
                    </a>
                </p>
            </div>
        </div>
    );
}

// Content Component
interface ContentProps {
    data: StatusData | null;
    loading: boolean;
    error: string | null;
    onRefetch: () => void;
    onShowLog: (monitorId: string, logEntries: string[]) => void;
}

function Content({ data, loading, error, onRefetch, onShowLog }: ContentProps) {
    const [activeTab, setActiveTab] = useState<string>(() => {
        // Get initial tab from URL hash
        const hash = window.location.hash.slice(1);
        if (hash.startsWith('visualization/')) {
            return 'visualization';
        }
        return hash === 'monitors' ? 'monitors' : 'visualization';
    });

    const [fullscreenVisualization, setFullscreenVisualization] = useState<string | null>(() => {
        const hash = window.location.hash.slice(1);
        if (hash.startsWith('visualization/')) {
            return hash.replace('visualization/', '');
        }
        return null;
    });

    useEffect(() => {
        // Update URL when tab changes
        if (fullscreenVisualization) {
            window.location.hash = `visualization/${fullscreenVisualization}`;
        } else {
            window.location.hash = activeTab;
        }
    }, [activeTab, fullscreenVisualization]);

    useEffect(() => {
        // Listen for URL changes
        const handleHashChange = () => {
            const hash = window.location.hash.slice(1);
            if (hash.startsWith('visualization/')) {
                const vizName = hash.replace('visualization/', '');
                setFullscreenVisualization(vizName);
                setActiveTab('visualization');
            } else if (hash === 'monitors') {
                setActiveTab('monitors');
                setFullscreenVisualization(null);
            } else {
                setActiveTab('visualization');
                setFullscreenVisualization(null);
            }
        };

        window.addEventListener('hashchange', handleHashChange);
        return () => window.removeEventListener('hashchange', handleHashChange);
    }, []);

    const handleTabChange = (tab: string) => {
        setActiveTab(tab);
        setFullscreenVisualization(null);
    };

    const handleFullscreenVisualization = (visualizationTitle: string) => {
        const urlSafeId = createUrlSafeId(visualizationTitle);
        setFullscreenVisualization(urlSafeId);
    };

    const handleCloseFullscreen = () => {
        setFullscreenVisualization(null);
    };

    // Show error state if there's an error and no data
    if (error && !data) {
        return (
            <div className="content">
                <div className="error">
                    <h2>Error loading status</h2>
                    <p>{error}</p>
                    <button className="refresh-button" onClick={onRefetch}>
                        Retry
                    </button>
                </div>
            </div>
        );
    }

    return (
        <>
            {fullscreenVisualization ? (
                <FullscreenVisualization 
                    data={data}
                    visualizationName={fullscreenVisualization}
                    onShowLog={onShowLog}
                    onClose={handleCloseFullscreen}
                />
            ) : (
                <>
                    <TabNavigation activeTab={activeTab} onTabChange={handleTabChange} />
                    {activeTab === 'visualization' ? (
                        <VisualizationTab 
                            data={data} 
                            onShowLog={onShowLog}
                            onFullscreen={handleFullscreenVisualization}
                        />
                    ) : (
                        <MonitorsTab 
                            data={data}
                            loading={loading}
                            onRefetch={onRefetch}
                            onShowLog={onShowLog}
                        />
                    )}
                </>
            )}
        </>
    );
}

// Main App Component
function App(): JSX.Element {
    const { data, loading, error, refetch } = useStatusData();
    const { themeMode, cycleTheme } = useThemeToggle();
    const [logModal, setLogModal] = useState<{
        isOpen: boolean;
        monitorId: string;
        logEntries: string[];
    }>({
        isOpen: false,
        monitorId: '',
        logEntries: []
    });

    const handleShowLog = (monitorId: string, logEntries: string[]) => {
        setLogModal({
            isOpen: true,
            monitorId,
            logEntries
        });
    };

    const handleCloseLog = () => {
        setLogModal({
            isOpen: false,
            monitorId: '',
            logEntries: []
        });
    };

    const getHeaderTitle = () => {
        if (loading && !data) return 'Stylus Monitor';
        return data?.config.ui.title || 'Stylus Monitor';
    };

    const getHeaderDescription = () => {
        if (loading && !data) return 'Loading...';
        if (error) return data?.config.ui.description || 'Error';
        return data?.config.ui.description || 'Monitor Dashboard';
    };

    return (
        <div className="container">
            <Header 
                title={getHeaderTitle()}
                description={getHeaderDescription()}
                themeMode={themeMode}
                onThemeToggle={cycleTheme}
            />
            <Content 
                data={data}
                loading={loading}
                error={error}
                onRefetch={refetch}
                onShowLog={handleShowLog}
            />
            <LogModal
                isOpen={logModal.isOpen}
                onClose={handleCloseLog}
                monitorId={logModal.monitorId}
                logEntries={logModal.logEntries}
            />
        </div>
    );
}

// Render the app
console.log('Welcome to Stylus!');
const root = createRoot(document.getElementById("root")!);
root.render(<App />); 
