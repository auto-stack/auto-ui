// Type Definitions

export interface ViewerControl {
    status: string;
}

// API Functions

export async function viewer_health(): Promise<string> {
    const response = await fetch(`/api/viewer/health`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function open_file(path: string): Promise<string> {
    const response = await fetch(`/api/viewer/open-file`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function current_uri(session: string): Promise<string> {
    const response = await fetch(`/api/viewer/current-uri?session=${encodeURIComponent(session)}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function close_session(session: string): Promise<boolean> {
    const response = await fetch(`/api/viewer/close`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ session }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}
