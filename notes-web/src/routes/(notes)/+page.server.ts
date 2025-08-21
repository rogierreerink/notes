import type { PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ parent }) => {
	const parent_data = await parent();

	const first_note = parent_data.notes[0];
	if (first_note) {
		redirect(303, `/${parent_data.notes[0].id}`);
	}

	return {};
};
