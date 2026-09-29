import { createContext } from 'svelte';

export const [getFormSubmissionFn, setFormSubmissionFn] = createContext<() => Promise<void>>();
