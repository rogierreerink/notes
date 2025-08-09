import { fail, redirect, type ActionFailure } from '@sveltejs/kit';
import type { Actions } from './$types';
import { setUserPassword } from '$lib/api/users';

export type Error = {
	message: string;
};

export const actions = {
	default: async ({
		cookies,
		request,
		fetch
	}): Promise<ActionFailure<Error>> => {
		const userId = cookies.get('userId');
		if (!userId) {
			return redirect(303, '/signin');
		}

		const payload = await request.formData();

		const password = payload.get('password')?.toString();
		if (!password || password.length < 6) {
			return fail(400, {
				message: 'password must contain at least 6 characters'
			});
		}

		const password_result = await setUserPassword(fetch, userId, {
			password
		});

		if (!password_result.ok) {
			return fail(500, {
				message: 'something went wrong'
			});
		}

		redirect(303, '/');
	}
} satisfies Actions;
