import { createUser } from '$lib/api/users';
import { fail, redirect, type ActionFailure } from '@sveltejs/kit';
import type { Actions } from './$types';
import { dev } from '$app/environment';

export type Error = {
	username?: string;
	message: string;
};

export const actions = {
	default: async ({
		request,
		cookies,
		fetch
	}): Promise<ActionFailure<Error>> => {
		const payload = await request.formData();

		const username = payload.get('username')?.toString();
		if (!username || username.length < 6) {
			return fail(400, {
				username,
				message: 'username must contain at least 6 characters'
			});
		}

		const user_result = await createUser(fetch, {
			username
		});

		if (!user_result.ok) {
			return fail(500, {
				message: 'something went wrong'
			});
		}

		cookies.set('sessionToken', user_result.data.session.token, {
			path: '/',
			httpOnly: true,
			sameSite: 'strict',
			secure: !dev
		});

		redirect(303, '/signup/password');
	}
} satisfies Actions;
