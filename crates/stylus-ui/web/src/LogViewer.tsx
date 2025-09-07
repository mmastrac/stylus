import { useState, useEffect } from "react";

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

    useEffect(() => {
        if (!isOpen || !monitorId) {
            return;
        }

        const fetchLogs = async () => {
            setLoading(true);
            setError(null);
            setLogEntries([]);

            try {
                const response = await fetch(`/log/${encodeURIComponent(monitorId)}`);
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                const logs = (await response.text()).split('\n');
                setLogEntries(logs);
            } catch (err) {
                setError(err instanceof Error ? err.message : 'Failed to fetch logs');
                setLogEntries([]);
            } finally {
                setLoading(false);
            }
        };

        fetchLogs();
    }, [isOpen, monitorId]);

    if (!isOpen) return null;

    return (
        <div className="log-modal-overlay" onClick={onClose}>
            <div className="log-modal" onClick={(e) => e.stopPropagation()}>
                <div className="log-modal-header">
                    <h2>{monitorId} log</h2>
                    <button className="log-modal-close" onClick={onClose}>
                        ×
                    </button>
                </div>
                <div className="log-modal-content">
                    {loading ? (
                        <p>Loading...</p>
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