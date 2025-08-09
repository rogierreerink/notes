import { getNoteById } from '$lib/api/notes';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { searchNotes } from '$lib/api/notes';

export const load: PageServerLoad = async ({ fetch, params }) => {
	const notes = await searchNotes(fetch);
	if (!notes.ok) {
		error(404, 'notes could not be fetched');
	}

	const note = await getNoteById(fetch, params.note_id);
	if (!note.ok) {
		error(404, 'note could not be found');
	}

	return {
		notes: notes.data.data,
		note: note.data
	};
};
