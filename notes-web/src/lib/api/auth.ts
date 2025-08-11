import { api } from './client';

export type User = {
	id: string;
};

export type Session = {
	token: string;
};

export type AuthenticationMethod = {
	method: 'password';
	username: string;
	password: string;
};

export async function authenticate(
	fetcher: typeof fetch,
	method: AuthenticationMethod
) {
	return await api<{
		user: User;
		session: Session;
	}>(fetcher, '/auth', {
		method: 'POST',
		headers: {
			'content-type': 'application/json'
		},
		body: JSON.stringify(method)
	});
}
