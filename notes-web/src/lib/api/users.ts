import { api } from './client';

export type Session = {
	token: string;
};

export type User = {
	id: string;
	username: string;
};

export type CreateUser = {
	username: string;
};

export async function createUser(fetcher: typeof fetch, user: CreateUser) {
	return await api<{
		user: User;
		session: Session;
	}>(fetcher, '/users', {
		method: 'POST',
		headers: {
			'content-type': 'application/json'
		},
		body: JSON.stringify(user)
	});
}

export type CreatePassword = {
	password: string;
};

export async function setUserPassword(
	fetcher: typeof fetch,
	userId: string,
	password: CreatePassword
) {
	return await api<void>(fetcher, `/users/${userId}/password`, {
		method: 'PUT',
		headers: {
			'content-type': 'application/json'
		},
		body: JSON.stringify(password)
	});
}
