<script lang="ts">
  import { tick } from "svelte";
  import Badge from "./Badge.svelte";

  let {
    parentForm,
    fieldName,
    options,
    targetArray = $bindable(),
  }: {
    parentForm: HTMLFormElement;
    fieldName: string;
    options: { optValue: string; displayText: string }[];
    targetArray: string[];
  } = $props();
  let value = $state("");

  function handleAddition() {
    if (value !== "") {
      targetArray.push(value);
      value = "";
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      handleAddition();
    }
  }
</script>

<label class="label text-base-content font-bold" for={fieldName}>{
  fieldName
}</label>
<div class="join">
  <input
    list={`${fieldName}-options`}
    autocomplete="off"
    id={fieldName}
    bind:value
    onkeydown={onKeydown}
    type="text"
    class="input join-item grow"
  />
  <datalist id={`${fieldName}-options`}>
    {#each options as { optValue, displayText }}
      <option value={optValue}>{displayText}</option>
    {/each}
  </datalist>
  <button
    aria-label="add item"
    onclick={handleAddition}
    class="btn btn-primary join-item"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 16 16"
      fill="currentColor"
      class="size-4"
    >
      <path
        fill-rule="evenodd"
        d="M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14Zm.75-10.25v2.5h2.5a.75.75 0 0 1 0 1.5h-2.5v2.5a.75.75 0 0 1-1.5 0v-2.5h-2.5a.75.75 0 0 1 0-1.5h2.5v-2.5a.75.75 0 0 1 1.5 0Z"
        clip-rule="evenodd"
      />
    </svg>
  </button>
</div>
<div class="flex flex-row flex-wrap gap-1">
  {#each targetArray as item, i}
    <Badge badgeText={item}>
      <button
        aria-label="remove item"
        onclick={async () => {
          targetArray.splice(i, 1);
          await tick();
          parentForm.requestSubmit();
        }}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="size-6"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="m9.75 9.75 4.5 4.5m0-4.5-4.5 4.5M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
          />
        </svg>
      </button>
    </Badge>
  {/each}
</div>
