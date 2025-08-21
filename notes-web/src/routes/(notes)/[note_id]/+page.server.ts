import { deleteNoteById, getNoteById } from '$lib/api/notes';
import { error, fail } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { Actions } from '@sveltejs/kit';
import { redirect } from '@sveltejs/kit';
import type { ActionFailure } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, params }) => {
	const note = await getNoteById(fetch, params.note_id);
	if (!note.ok) {
		error(404, 'note could not be found');
	}

	return {
		note: note.data
	};
};

export type Error = {
	message: string;
};

export const actions = {
	delete: async ({ params, locals, fetch }): Promise<ActionFailure<Error>> => {
		if (!locals.session) {
			redirect(303, '/signup');
		}

		const id = params.note_id;
		if (!id) {
			error(400, 'note id cannot be empty');
		}

		const delete_result = await deleteNoteById(fetch, id);
		if (!delete_result.ok) {
			return fail(500, {
				message: 'something went wrong'
			});
		}

		redirect(303, '/');
	}
} satisfies Actions;
