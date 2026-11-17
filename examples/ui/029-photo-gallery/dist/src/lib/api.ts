// Type Definitions

export interface GalleryInfo {
    status: string;
    version: string;
}

// API Functions

export async function status(): Promise<GalleryInfo[]> {
    const response = await fetch(`/api/gallery/status`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
}
