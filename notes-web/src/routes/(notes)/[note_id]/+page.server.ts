import { getNoteById } from '$lib/api/notes';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params }) => {
	const note = await getNoteById(fetch, params.note_id);
	if (!note.ok) {
		error(404, 'note could not be found');
	}

	return {
		note: note.data
	};
};
