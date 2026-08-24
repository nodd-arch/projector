import { writable } from 'svelte/store';

export const translationId = writable(1);
export const projectedRef = writable(null); // { bookid, chapternumber, versenumber }
export const isPanicked = writable(false);
export const livePreview = writable(null); // { verse, translation_abbr }
export const mode = writable('search'); // 'search' | 'browse' | 'history'
export const currentSessionId = writable(Date.now());
