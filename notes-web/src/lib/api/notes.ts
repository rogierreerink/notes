import { api } from './client';

export type Notes = {
	data: {
		id: string;
		title: string;
	}[];
};

export function searchNotes(fetcher: typeof fetch) {
	return api<Notes>(fetcher, '/notes');
}

export type Note = {
	id: string;
	title: string;
	markdown: string;
};

export function getNoteById(fetcher: typeof fetch, noteId: string) {
	return api<Note>(fetcher, `/notes/${noteId}`);
}

export type UpsertNote = {
	id: string;
	markdown: string;
};

export function upsertNoteById(fetcher: typeof fetch, note: UpsertNote) {
	return api<void>(fetcher, `/notes/${note.id}`, {
		method: 'PUT',
		headers: {
			'content-type': 'application/json'
		},
		body: JSON.stringify(note)
	});
}
