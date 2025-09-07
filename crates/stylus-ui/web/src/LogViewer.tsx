import { useState, useEffect, useRef } from "react";

// Log Modal Component
interface LogModalProps {
    isOpen: boolean;
    onClose: () => void;
    monitorId: string;
}

export function LogModal({ isOpen, onClose, monitorId }: LogModalProps) {
    const [logEntries, setLogEntries] = useState<string[]>([]);
    const [loading, setLoading] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const [showModal, setShowModal] = useState<boolean>(false);
    const [hasContentLoaded, setHasContentLoaded] = useState<boolean>(false);
    const timeoutRef = useRef<number | null>(null);

    useEffect(() => {
        if (!isOpen || !monitorId) {
            setShowModal(false);
            setHasContentLoaded(false);
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
                timeoutRef.current = null;
            }
            return;
        }

        // Reset states
        setLoading(true);
        setError(null);
        setLogEntries([]);
        setHasContentLoaded(false);

        // Set up 100ms delay
        timeoutRef.current = window.setTimeout(() => {
            setShowModal(true);
            timeoutRef.current = null;
        }, 100);

        const fetchLogs = async () => {
            try {
                const response = await fetch(`/log/${encodeURIComponent(monitorId)}`);
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                const logs = (await response.text()).split('\n');
                setLogEntries(logs);
                setHasContentLoaded(true);
                
                // If delay hasn't passed yet, show modal immediately
                if (timeoutRef.current) {
                    clearTimeout(timeoutRef.current);
                    timeoutRef.current = null;
                    setShowModal(true);
                }
            } catch (err) {
                setError(err instanceof Error ? err.message : 'Failed to fetch logs');
                setLogEntries([]);
                setHasContentLoaded(true);
                
                // If delay hasn't passed yet, show modal immediately with error
                if (timeoutRef.current) {
                    clearTimeout(timeoutRef.current);
                    timeoutRef.current = null;
                    setShowModal(true);
                }
            } finally {
                setLoading(false);
            }
        };

        fetchLogs();

        // Cleanup timeout on unmount
        return () => {
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
                timeoutRef.current = null;
            }
        };
    }, [isOpen, monitorId]);

    if (!isOpen || !showModal) return null;

    return (
        <div className="log-modal-overlay" onClick={onClose}>
            <div className={`log-modal ${hasContentLoaded ? 'content-loaded' : 'loading'}`} onClick={(e) => e.stopPropagation()}>
                <div className="log-modal-header">
                    <h2>{monitorId} log</h2>
                    <button className="log-modal-close" onClick={onClose}>
                        ×
                    </button>
                </div>
                <div className="log-modal-content">
                    {loading && !hasContentLoaded ? (
                        <div className="log-loading">
                            <div className="loading-spinner"></div>
                            <p>Loading...</p>
                        </div>
                    ) : error ? (
                        <p>Error loading logs: {error}</p>
                    ) : logEntries.length > 0 ? (
                        <div className="log-entries">
                            {logEntries.map((entry, index) => (
                                <div key={index} className="log-entry">
                                    {entry}
                                </div>
                            ))}
                        </div>
                    ) : (
                        <p>No log entries available.</p>
                    )}
                </div>
            </div>
        </div>
    );
} 