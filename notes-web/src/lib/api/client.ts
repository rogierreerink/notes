export const BASE_URL = new URL('http://localhost:3123/api');

export type Result<T> =
	| {
			ok: true;
			data: T;
	  }
	| {
			ok: false;
			error: Error;
	  };

export async function api<T>(
	fetcher: typeof fetch,
	path: string,
	options: RequestInit = {}
): Promise<Result<T>> {
	try {
		const res = await fetcher(`${BASE_URL}${path}`, options);

		if (!res.ok) {
			throw new Error(`api error: ${res.status}`);
		}

		return {
			ok: true,
			data: await res.json()
		};
	} catch (e) {
		return {
			ok: false,
			error: e as Error
		};
	}
}
