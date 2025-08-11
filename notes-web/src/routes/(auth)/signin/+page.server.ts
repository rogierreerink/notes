import type { Actions } from './$types';

export const actions = {
	default: async (event) => {
		console.log('Login action triggered', event);
	}
} satisfies Actions;
