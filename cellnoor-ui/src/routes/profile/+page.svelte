<script lang="ts">
  import { enhance } from "$app/forms";
  import { DATETIME_FORMATTER } from "$lib/date.js";

  const { data, form } = $props();
  const {
    apiTokens,
    user: { name: userName, email, image, userId },
    today,
    oneYearFromNow,
  } = $derived(data);
  let apiKeysDialogBox: HTMLDialogElement;
</script>

<div class="min-h-1/2 mx-auto flex flex-col items-center w-fit">
  <div class="avatar">
    <img
      class="rounded-full"
      src={image}
      alt="profile"
    />
  </div>
  <h1 class="text-4xl font-bold">{userName}</h1>
  <p class="text-xl font-bold">{email}</p>
  <div class="divider"></div>
  <button
    class="btn btn-primary btn-outline"
    onclick={async () => {
      apiKeysDialogBox.showModal();
    }}
  >
    API Tokens
  </button>
  <dialog bind:this={apiKeysDialogBox} class="modal">
    <div class="modal-box max-w-full xl:max-w-1/2 lg:max-w-3/4">
      <table class="table">
        <thead>
          <tr>
            <th>API token name</th>
            <th>Description</th>
            <th>Created at</th>
            <th>Expires on</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each apiTokens as { jti, name, description, iat, exp }}
            <tr>
              <td>
                {name}
              </td>
              <td>
                {description}
              </td>
              <td>
                {
                  DATETIME_FORMATTER.format(
                    iat,
                  )
                }
              </td>
              <td>
                {DATETIME_FORMATTER.format(exp)}
              </td>
              <td>
                <form method="post" use:enhance action="?/deleteApiToken">
                  <input name="jti" value={jti} type="hidden" />
                  <button class="btn btn-error">Delete</button>
                </form>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if form}
        <div class="wrap-anywhere py-1 text-left">
          {#if form?.apiToken}
            Your new API token is <code class="font-bold">{
              form?.apiToken
            }</code>. You will not be able to view this token after leaving or
            refreshing this page. Store this token securely.
          {:else if form?.error}
            {form.error}
          {/if}
        </div>
      {/if}

      <div class="modal-action flex flex-row justify-evenly">
        <form method="post" use:enhance action="?/createApiToken">
          <label class="input">
            <span class="label">API token name</span>
            <input name="name" type="text" />
          </label>
          <label class="input">
            <span class="label">Expires on</span>
            <input
              name="expiresOn"
              type="date"
              min={today}
              max={oneYearFromNow}
              required
            />
          </label>
          <button class="btn btn-success">Create new API token</button>
        </form>
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
