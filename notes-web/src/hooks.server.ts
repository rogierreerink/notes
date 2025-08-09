import { BASE_URL } from '$lib/api/client';
import type { Handle, HandleFetch } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	// Get session token from cookies
	event.locals.sessionToken = event.cookies.get('sessionToken');

	return resolve(event);
};

export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
	const url = new URL(request.url);

	// Set the authorization header of requests to the data store
	if (
		event.locals.sessionToken &&
		url.hostname === BASE_URL.hostname &&
		url.port === BASE_URL.port
	) {
		request.headers.set('authorization', `Bearer ${event.locals.sessionToken}`);
	}

	return fetch(request);
};
