<script lang="ts">
  import { authClient } from "$lib/auth-client";

  const session = authClient.useSession();
  let apiKeyPrefixes: string[] = $state([]);
  let newApiKey: string | undefined = $state();
  let developerToolsDialogBox: HTMLDialogElement;

  async function getApiKeyPrefixes() {
    const response = await fetch("/ui-api/api-keys");
    const json = await response.json();

    apiKeyPrefixes = json.apiKeyPrefixes;
  }

  async function createApiKey() {
    const response = await fetch("/ui-api/api-keys", { method: "POST" });
    const json = await response.json();

    await getApiKeyPrefixes();
    newApiKey = json.apiKey;
  }

  async function deleteApiKey(apiKeyPrefix: string) {
    await fetch("/ui-api/api-keys", {
      body: JSON.stringify({
        apiKeyPrefix,
      }),
      method: "DELETE",
    });

    await getApiKeyPrefixes();
  }
</script>

<div class="h-screen">
  <div class="hero bg-base h-1/2">
    <div class="hero-content text-center">
      <div>
        {#if $session.data?.user.name}
          <div class="avatar">
            <img
              class="rounded-full"
              src={$session.data.user.image}
              alt="profile"
            />
          </div>
        {/if}
        <h1 class="text-4xl font-bold">{$session.data?.user.name}</h1>
        <p class="text-2xl font-bold">{$session.data?.user.email}</p>
        <div class="divider"></div>
        <button
          class="btn btn-primary"
          onclick={async () => {
            developerToolsDialogBox.showModal();
            await getApiKeyPrefixes();
          }}
        >
          Developer tools
        </button>
        <dialog bind:this={developerToolsDialogBox} class="modal">
          <div class="modal-box">
            {#if apiKeyPrefixes.length > 0}
              <p class="font-bold text-lg">API keys</p>
              <table class="table">
                <thead>
                  <tr>
                    <th>API Key Prefix (hex-encoded)</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each apiKeyPrefixes as apiKeyPrefix}
                    <tr>
                      <td>
                        {apiKeyPrefix}
                      </td>
                      <td>
                        <button
                          onclick={async () => {
                            await deleteApiKey(apiKeyPrefix);
                          }}
                          class="btn btn-error"
                        >
                          Delete
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
            <button onclick={createApiKey} class="btn btn-accent">
              Create new API Key
            </button>
            {#if newApiKey}
              <div class="wrap-anywhere py-1 text-left">
                Your new API key is <span class="font-bold">{newApiKey}</span>.
                You will not be able to view this API key after leaving or
                refreshing this page.
              </div>
            {/if}
            <div class="modal-action">
              <form method="dialog">
                <button class="btn btn-secondary">Close</button>
              </form>
            </div>
          </div>
          <form method="dialog" class="modal-backdrop">
            <button>close</button>
          </form>
        </dialog>
      </div>
    </div>
  </div>
</div>
