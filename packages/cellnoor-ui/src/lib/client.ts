import { error } from '@sveltejs/kit';
import { createCellnoorClient } from 'cellnoor-client';

export const cellnoorClient = createCellnoorClient({ baseUrl: '/api' });

export function unwrap<T, E>(result: { data?: T; error?: E; response: Response }): T {
	if (result.data === undefined) {
		error(result.response.status, 'something went wrong');
	}

	return result.data;
}
