// See https://svelte.dev/docs/kit/types#app.d.ts

import type { auth } from "$lib/auth";

// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // Technically, there are a couple routes where we don't have a session, but TypeScript is annoying
    interface Locals {
      user: typeof auth.$Infer.Session.user;
    }
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
