import { getNoteById, upsertNoteById } from '$lib/api/notes';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ActionFailure, Actions } from '@sveltejs/kit';
import { redirect } from '@sveltejs/kit';
import { fail } from '@sveltejs/kit';

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
	default: async ({
		request,
		params,
		locals,
		fetch
	}): Promise<ActionFailure<Error>> => {
		if (!locals.session) {
			redirect(303, '/signup');
		}

		const id = params.note_id;
		if (!id) {
			error(400, 'note id cannot be empty');
		}

		const payload = await request.formData();

		const markdown = payload.get('markdown')?.toString();
		if (!markdown) {
			return fail(400, {
				message: 'markdown cannot be empty'
			});
		}

		const upsert_result = await upsertNoteById(fetch, {
			id,
			markdown
		});
		if (!upsert_result.ok) {
			return fail(500, {
				message: 'something went wrong'
			});
		}

		redirect(303, `/${params.note_id}`);
	}
} satisfies Actions;
