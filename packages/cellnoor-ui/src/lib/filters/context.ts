import { createContext } from 'svelte';

export const [getFormContext, setFormContext] = createContext<HTMLFormElement>();
