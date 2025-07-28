// Log Modal Component
interface LogModalProps {
    isOpen: boolean;
    onClose: () => void;
    monitorId: string;
    logEntries: string[];
}

export function LogModal({ isOpen, onClose, monitorId, logEntries }: LogModalProps) {
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
                    {logEntries.length > 0 ? (
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