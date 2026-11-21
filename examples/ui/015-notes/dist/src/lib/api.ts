// Type Definitions

export interface Note {
    id: number;
    title: string;
    body: string;
    time: string;
    pinned: boolean;
    tags: string[];
    folder: string;
}

// API Functions

export async function list_notes(): Promise<Note[]> {
    const response = await fetch(`/api/notes`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function get_note(id: number): Promise<Note | null> {
    const response = await fetch(`/api/notes/${id}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function create_note(title: string, body: string, folder: string): Promise<Note> {
    const response = await fetch(`/api/notes`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, body, folder }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function update_note(id: number, title: string, body: string): Promise<Note | null> {
    const response = await fetch(`/api/notes/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, body }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function delete_note(id: number): Promise<boolean> {
    const response = await fetch(`/api/notes/${id}`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function toggle_pin(id: number): Promise<Note | null> {
    const response = await fetch(`/api/notes/${id}/pin`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function update_tags(id: number, tags: string[]): Promise<Note | null> {
    const response = await fetch(`/api/notes/${id}/tags`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tags }),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}

export async function search_notes(query: string): Promise<Note[]> {
    const response = await fetch(`/api/notes/search?query=${encodeURIComponent(query)}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}
