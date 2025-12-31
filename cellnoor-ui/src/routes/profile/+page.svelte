<script lang="ts">
  import { authClient } from "$lib/auth-client";
  import { DATETIME_FORMATTER } from "$lib/date.js";

  const session = authClient.useSession();
  const { data } = $props();
  // svelte-ignore state_referenced_locally
  let apiKeyPrefixes = $state(data.apiKeyPrefixes);
  let newApiKey: string | undefined = $state();
  let apiKeysDialogBox: HTMLDialogElement;

  async function createApiKey() {
    const response = await fetch("/ui-api/api-keys", { method: "POST" });
    const json = await response.json();

    const { api_key, created_at } = json;
    if (api_key) {
      // The apiKey is returned as a hex-encoded, meaning each byte is represented by two characters
      apiKeyPrefixes.push({
        prefix: api_key.slice(0, data.apiKeyPrefixLength * 2),
        created_at: new Date(created_at),
      });
      newApiKey = api_key;
    }
  }

  async function deleteApiKey(apiKeyPrefix: string, idx: number) {
    const response = await fetch("/ui-api/api-keys", {
      body: JSON.stringify({
        apiKeyPrefix,
      }),
      method: "DELETE",
    });

    const { error } = await response.json();

    if (!error) {
      apiKeyPrefixes.splice(idx, 1);
    }
  }
</script>

<div class="min-h-1/2 mx-auto flex flex-col items-center w-fit">
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
  <p class="text-xl font-bold">{$session.data?.user.email}</p>
  <div class="divider"></div>
  <button
    class="btn btn-primary btn-outline"
    onclick={async () => {
      apiKeysDialogBox.showModal();
    }}
  >
    API Keys
  </button>
  <dialog bind:this={apiKeysDialogBox} class="modal">
    <div class="modal-box max-w-full xl:max-w-1/2 lg:max-w-3/4">
      <table class="table">
        <thead>
          <tr>
            <th>API key prefix (hex-encoded)</th>
            <th>Created at</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each apiKeyPrefixes as { prefix, created_at }, i}
            <tr>
              <td>
                {prefix}
              </td>
              <td>
                {
                  DATETIME_FORMATTER.format(
                    created_at,
                  )
                }
              </td>
              <td>
                <button
                  onclick={async () => {
                    await deleteApiKey(prefix, i);
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
      {#if newApiKey}
        <div class="wrap-anywhere py-1 text-left">
          Your new (hex-encoded) API key is <code class="font-bold">{
            newApiKey
          }</code>. You will not be able to view this API key after leaving or
          refreshing this page.
        </div>
      {/if}
      <div class="modal-action flex flex-row justify-evenly">
        <button onclick={createApiKey} class="btn btn-success">
          Create new API key
        </button>
        <form method="dialog">
          <button class="btn btn-secondary btn-outline">Close</button>
        </form>
      </div>
    </div>
    <form method="dialog" class="modal-backdrop">
      <button>close</button>
    </form>
  </dialog>
</div>
