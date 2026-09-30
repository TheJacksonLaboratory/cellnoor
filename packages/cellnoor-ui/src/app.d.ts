import type {
	AuthError,
	CreateChromiumDatasetError,
	DbError,
	IndexSetError,
	JsonRejection,
	PersonError,
	UpdateChromiumDatasetError
} from 'cellnoor-client/cellnoor-types.js';

// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		interface Error {
			message: string;
			error?:
				| AuthError
				| CreateChromiumDatasetError
				| DbError
				| IndexSetError
				| JsonRejection
				| PersonError
				| UpdateChromiumDatasetError;
		}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
