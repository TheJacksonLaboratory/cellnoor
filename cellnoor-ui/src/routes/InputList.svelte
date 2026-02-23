<!-- TODO: this is extremely repetetive and IDK how to consolidate it with the other `InputList` component -->
<script lang="ts">
  import { tick } from "svelte";
  import Badge from "./Badge.svelte";

  let {
    parentForm,
    fieldName,
    targetArray = $bindable(),
  }: {
    parentForm: HTMLFormElement;
    fieldName: string;
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
    autocomplete="off"
    id={fieldName}
    bind:value
    onkeydown={onKeydown}
    type="text"
    class="input join-item grow"
  />
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
          viewBox="0 0 16 16"
          fill="currentColor"
          class="size-4"
        >
          <path
            fill-rule="evenodd"
            d="M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14Zm2.78-4.22a.75.75 0 0 1-1.06 0L8 9.06l-1.72 1.72a.75.75 0 1 1-1.06-1.06L6.94 8 5.22 6.28a.75.75 0 0 1 1.06-1.06L8 6.94l1.72-1.72a.75.75 0 1 1 1.06 1.06L9.06 8l1.72 1.72a.75.75 0 0 1 0 1.06Z"
            clip-rule="evenodd"
          />
        </svg>
      </button>
    </Badge>
  {/each}
</div>
