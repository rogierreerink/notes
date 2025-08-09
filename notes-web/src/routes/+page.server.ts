import { error, redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { searchNotes } from '$lib/api/notes';

export const load: PageServerLoad = async ({ fetch }) => {
	const notes = await searchNotes(fetch);
	if (!notes.ok) {
		error(404, 'notes could not be fetched');
	}

	const first_note = notes.data.data[0];
	if (!first_note) {
		error(404, 'not notes found');
	}

	redirect(303, `/${first_note.id}`);
};
