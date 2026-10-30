import { error } from '@sveltejs/kit';
import { invalidate } from '$app/navigation';
import { createCellnoorClient } from 'cellnoor-client';
import { DATE_FORMATTER, formatEnum } from './format.ts';
import { toast } from './toasts.svelte.ts';
import type {
	AuthError,
	DbError,
	JsonRejection,
	PersonError
} from 'cellnoor-client/cellnoor-types.ts';

export const cellnoorClient = createCellnoorClient({ baseUrl: '/api' });

export function unwrap<T, E extends App.Error['error']>(result: {
	data?: T;
	error?: E;
	response: Response;
}): T {
	if (result.data === undefined) {
		error(result.response.status, 'Something went wrong', {
			error: result.error
		});
	}

	return result.data;
}

export type Mutation = Promise<{ error?: DbError | AuthError | PersonError | JsonRejection }>;

export function errorMessage(error: DbError | AuthError | PersonError | JsonRejection) {
	const isJsonRejection = (e: typeof error): e is JsonRejection =>
		(e as JsonRejection).status !== undefined;

	if (isJsonRejection(error)) {
		return error.message;
	}

	switch (error.type) {
		case 'resource_not_found':
			return "Item not found (or you don't have access to it)";
		case 'invalid_reference':
			return `Invalid ${formatEnum(error.referencing_field)} for ${formatEnum(error.referencing_resource)}`;
		case 'data_constraint':
			return error.message;
		case 'permission_denied':
			return 'Permission denied';
		case 'expired_api_key':
			return `API key expired on ${DATE_FORMATTER.format(new Date(error.expired_at))}`;
		case 'invalid_auth_token':
		case 'no_auth_found':
			return 'You are not signed in';
		case 'invalid_email':
			return `${error.email} is not a valid email address`;
		case 'other':
			return 'Something went wrong';
	}

	return 'Something went wrong';
}

export const MUTABLE_DATA = 'cellnoor:mutable-data';

export async function mutate(request: Mutation, success = 'Saved') {
	const result = await request.catch(() => null);

	if (!result) {
		toast('Could not reach the server', false);
		return false;
	}

	if (result.error) {
		toast(errorMessage(result.error), false);
		return false;
	}

	await invalidate(MUTABLE_DATA);
	toast(success, true);
	return true;
}
