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

export async function open_directory(path: string): Promise<string> {
    const response = await fetch(`/api/viewer/open-directory`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function open_path(path: string): Promise<string> {
    const response = await fetch(`/api/viewer/open-path`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function snapshot(session: string): Promise<string> {
    const response = await fetch(`/api/viewer/snapshot?session=${encodeURIComponent(session)}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
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

export async function names(session: string): Promise<string[]> {
    const response = await fetch(`/api/viewer/names?session=${encodeURIComponent(session)}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function navigate(session: string, delta: number): Promise<string> {
    const response = await fetch(`/api/viewer/navigate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ session, delta }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function request_view(session: string, index: number, viewport_width: number, viewport_height: number, quality: number): Promise<string> {
    const response = await fetch(`/api/viewer/request-view`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ session, index, viewport_width, viewport_height, quality }),
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

export async function image_stats(session: string): Promise<string> {
    const response = await fetch(`/api/viewer/stats?session=${encodeURIComponent(session)}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}
