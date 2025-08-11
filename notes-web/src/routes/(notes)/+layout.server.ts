import type { LayoutServerLoad } from './$types';
import { searchNotes } from '$lib/api/notes';
import { redirect } from '@sveltejs/kit';

export const load: LayoutServerLoad = async ({ fetch }) => {
	const notes = await searchNotes(fetch);
	if (!notes.ok) {
		redirect(303, '/signin');
	}

	return {
		notes: notes.data.data
	};
};
