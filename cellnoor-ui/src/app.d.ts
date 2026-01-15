// See https://svelte.dev/docs/kit/types#app.d.ts

import type { auth } from "./auth";

// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    interface Locals {
      session: typeof auth.$Infer.Session;
    }
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
