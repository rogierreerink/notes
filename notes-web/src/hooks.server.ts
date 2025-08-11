import { BASE_URL } from '$lib/api/client';
import type { Handle, HandleFetch } from '@sveltejs/kit';
import { decodeProtectedHeader } from 'jose';

export const handle: Handle = async ({ event, resolve }) => {
	// Get the session token from the cookies and decode non-sensitive data
	const sessionToken = event.cookies.get('sessionToken');
	if (sessionToken) {
		const header = decodeProtectedHeader(sessionToken);
		event.locals.session = {
			token: sessionToken,
			id: header.session_id as string,
			userId: header.user_id as string
		};
	}

	return resolve(event);
};

export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
	const url = new URL(request.url);

	// Set the authorization header in requests to the data store
	if (
		event.locals.session &&
		url.hostname === BASE_URL.hostname &&
		url.port === BASE_URL.port
	) {
		request.headers.set(
			'authorization',
			`Bearer ${event.locals.session.token}`
		);
	}

	return fetch(request);
};
