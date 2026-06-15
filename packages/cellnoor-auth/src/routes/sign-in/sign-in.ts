import { createAuthClient } from "better-auth/client";

const authClient = createAuthClient();

const redirectTo = window.location.search
  ? new URLSearchParams(window.location.search).get("redirect_to")
  : "/";

async function signInWithMicrosoft() {
  return await authClient.signIn.social({
    provider: "microsoft",
    callbackURL: redirectTo ?? "/",
  });
}

document.getElementById("microsoft")!.addEventListener(
  "click",
  signInWithMicrosoft,
);
