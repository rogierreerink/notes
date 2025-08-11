import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import { authenticate } from '$lib/api/auth';
import { dev } from '$app/environment';
import type { ActionFailure } from '@sveltejs/kit';

export type Error = {
	username?: string;
	message: string;
};

export const actions = {
	default: async ({
		request,
		fetch,
		cookies
	}): Promise<ActionFailure<Error>> => {
		const payload = await request.formData();

		const username = payload.get('username')?.toString();
		if (!username) {
			return fail(400, {
				message: 'username is required'
			});
		}

		const password = payload.get('password')?.toString();
		if (!password) {
			return fail(400, {
				message: 'password is required'
			});
		}

		const auth_result = await authenticate(fetch, {
			method: 'password',
			username,
			password
		});
		if (!auth_result.ok) {
			return fail(500, {
				username,
				message: 'authentication failed'
			});
		}

		cookies.set('userId', auth_result.data.user.id, {
			path: '/',
			httpOnly: true,
			sameSite: 'strict',
			secure: !dev
		});

		cookies.set('sessionToken', auth_result.data.session.token, {
			path: '/',
			httpOnly: true,
			sameSite: 'strict',
			secure: !dev
		});

		redirect(303, '/');
	}
} satisfies Actions;
