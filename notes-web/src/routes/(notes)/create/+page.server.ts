import { upsertNoteById } from '$lib/api/notes';
import type { ActionFailure, Actions } from '@sveltejs/kit';
import { redirect } from '@sveltejs/kit';
import { fail } from '@sveltejs/kit';
import { v4 as uuid_v4 } from 'uuid';

export type Error = {
	message: string;
};

export const actions = {
	default: async ({
		request,
		locals,
		fetch
	}): Promise<ActionFailure<Error>> => {
		if (!locals.session) {
			redirect(303, '/signup');
		}

		const payload = await request.formData();
		const markdown = payload.get('markdown')?.toString();
		if (!markdown) {
			return fail(400, {
				message: 'markdown cannot be empty'
			});
		}

		const id = uuid_v4();
		const upsert_result = await upsertNoteById(fetch, {
			id,
			markdown
		});
		if (!upsert_result.ok) {
			return fail(500, {
				message: 'something went wrong'
			});
		}

		redirect(303, `/${id}`);
	}
} satisfies Actions;
