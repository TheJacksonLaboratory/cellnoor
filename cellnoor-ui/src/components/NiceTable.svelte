<script lang="ts" generics="T">
  type ExtractDatum = (fieldName: string, row: T) => unknown;

  const {
    fieldNames,
    data,
    extractDatum,
  }: {
    fieldNames: string[];
    data: T[];
    extractDatum: ExtractDatum;
  } = $props();
</script>

<table class="table bg-base-200 rounded">
  <thead>
    <tr>
      {#each fieldNames as fieldName (fieldName)}
        <th class="text-neutral">{fieldName}</th>
      {/each}
    </tr>
    {#each data as row (row)}
      <tr>
        {#each fieldNames as fieldName (fieldName)}
          <td class="text-neutral font-normal">{extractDatum(fieldName, row)}</td>
        {/each}
      </tr>
    {/each}
  </thead>
</table>
