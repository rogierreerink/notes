import type { PageServerLoad } from './$types';
import { getNoteById } from '$lib/api/notes';
import { error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ parent, fetch }) => {
	const parent_data = await parent();

	const first_note = parent_data.notes[0];
	if (!first_note) {
		return {
			notes: parent_data.notes
		};
	}

	const note = await getNoteById(fetch, first_note.id);
	if (!note.ok) {
		error(404, 'failed to fetch note');
	}

	return {
		notes: parent_data.notes,
		note: note.data
	};
};
