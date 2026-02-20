<script lang="ts">
  let {
    fieldName,
    targetArray,
  }: { fieldName: string; targetArray: string[] } = $props();
  let value = $state("");

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      targetArray.push(value);
      value = "";
      console.log(targetArray);
    }
  }
</script>

<label class="label" for={fieldName}>{fieldName}</label>
<input
  id={fieldName}
  bind:value
  onkeydown={onKeydown}
  type="text"
  class="input"
  placeholder="Sample 1"
/>
