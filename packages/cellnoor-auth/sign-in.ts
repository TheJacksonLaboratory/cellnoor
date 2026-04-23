import { createAuthClient } from "better-auth/client";

const authClient = createAuthClient();

document.getElementById("sign-in-microsoft")?.addEventListener(
  "click",
  async () => {
    await authClient.signIn.social({
      provider: "microsoft",
      callbackURL: "/sign-in",
    });
  },
);
