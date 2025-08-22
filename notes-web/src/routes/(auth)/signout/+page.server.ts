import { deleteUserSession } from '$lib/api/users';
import { fail, redirect, type Actions } from '@sveltejs/kit';

export const actions = {
	default: async ({ locals, fetch }) => {
		if (!locals.session) {
			redirect(303, '/signin');
		}

		const delete_result = await deleteUserSession(
			fetch,
			locals.session.userId,
			locals.session.id
		);
		if (!delete_result.ok) {
			return fail(500, {
				message: 'something went wrong'
			});
		}

		redirect(303, '/signin');
	}
} satisfies Actions;
