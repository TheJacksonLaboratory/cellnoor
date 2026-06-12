<script lang="ts">
  import InputList from "../../components/InputList.svelte";
  import ChromiumDataset from "./ChromiumDataset.svelte";
  import Fieldset from "../../components/Fieldset.svelte";
  import SortAndLimit from "../../components/SortAndLimit.svelte";
  import type {
    ChromiumDatasetPredicate,
    ChromiumDatasetPredicateQuery,
    SpecimenPredicate,
    TenxAssayPredicate,
    OrderByChromiumDatasetField,
  } from "$lib/cellnoor-types";
  import { libraryTypeValues, sampleMultiplexingValues, speciesValues } from "$lib/cellnoor-types";
  import { isNonempty } from "$lib/query-utils";

  const { data } = $props();
  const { assays, datasets, projects, error } = $derived(data);

  let filterForm: HTMLFormElement | undefined = $state();

  let specimenNamePred = $state({ name: { trgm_any: [] } }) satisfies SpecimenPredicate;
  let speciesPred = $state({ species: { in: [] } }) satisfies SpecimenPredicate;
  let tissuePred = $state({ tissue: { trgm_any: [] } }) satisfies SpecimenPredicate;
  let projectPred = $state({
    project_id: { in: [] },
  }) satisfies SpecimenPredicate;
  let assayNamePred = $state({ name: { in: [] } }) satisfies TenxAssayPredicate;
  let assayMultiplexingPred = $state({
    sample_multiplexing: { in: [] },
  }) satisfies TenxAssayPredicate;
  let assayLibraryTypesPred = $state({
    library_types: { contains: [] },
  }) satisfies TenxAssayPredicate;

  const predicates = $derived([
    { specimen: specimenNamePred },
    { specimen: speciesPred },
    { specimen: tissuePred },
    { specimen: projectPred },
    { tenx_assay: assayNamePred },
    { tenx_assay: assayMultiplexingPred },
    { tenx_assay: assayLibraryTypesPred },
  ]) satisfies ChromiumDatasetPredicate[];

  let order_by = $state({ field: undefined, desc: undefined });

  const query = $derived({
    filter: {
      // @ts-expect-error I don't get why this is a type-error :)
      all_of: predicates.filter(isNonempty),
    },
    order_by,
  }) satisfies ChromiumDatasetPredicateQuery;

  const stringifiedQuery = $derived(JSON.stringify(query));

  let advancedQuery = $state("");
  let advancedQueryError = $derived.by(() => {
    if (!advancedQuery) {
      return "";
    }
    try {
      JSON.parse(advancedQuery);
      return "";
    } catch (error) {
      return error;
    }
  });

  function toLowercaseChoice(s: string) {
    return { label: s.replaceAll("_", " "), value: s };
  }

  const orderByValues = [
    { field: "delivered_at" },
    { field: "name" },
  ] satisfies OrderByChromiumDatasetField[];
  const orderByChoices = orderByValues.map(({ field }) => field).map(toLowercaseChoice);
</script>

<div class="drawer drawer-open">
  <input id="filter-drawer" type="checkbox" class="drawer-toggle" />
  <div class="drawer-content mt-4 px-4 flex flex-col items-stretch">
    {#if datasets && datasets.length != 0}
      <div class="mb-2 flex flex-row justify-between align-middle">
        <p class="text-lg">{datasets.length} results</p>
        <SortAndLimit
          bind:orderByField={order_by.field}
          bind:orderByDescending={order_by.desc}
          choices={orderByChoices}
          parentForm={filterForm!}
        />
      </div>
      {#each datasets as cd (cd.id)}
        <ChromiumDataset chromiumDataset={cd} />
      {/each}
    {:else if error}
      <p class="text-center text-error">Something went wrong</p>
    {:else}
      <p class="text-center">No matching Chromium datasets found</p>
    {/if}
  </div>
  <div class="drawer-side bg-base px-4 pt-4 border-r">
    <div class="lg:w-120 sm:w-80">
      <p class="font-bold text-lg px-2">Filter</p>
      <form bind:this={filterForm} action="?/search">
        <Fieldset name="Specimen Information">
          <InputList
            parentForm={filterForm}
            fieldName="Specimen Name"
            bind:values={specimenNamePred.name.trgm_any}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Species"
            choices={speciesValues.map(toLowercaseChoice)}
            bind:values={speciesPred.species.in}
          />
        </Fieldset>
        <Fieldset name="Lab Information">
          <InputList
            parentForm={filterForm}
            fieldName="Lab Name"
            choices={(projects || []).map((p) => toLowercaseChoice(p.name))}
            bind:values={projectPred.project_id.in}
          />
        </Fieldset>
        <Fieldset name="Assay Information">
          <InputList
            parentForm={filterForm}
            fieldName="Assay Name"
            choices={(assays || []).map((a) => toLowercaseChoice(a.name))}
            bind:values={assayNamePred.name.in}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Multiplexing"
            choices={sampleMultiplexingValues.map(toLowercaseChoice)}
            bind:values={assayMultiplexingPred.sample_multiplexing.in}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Library Types"
            choices={libraryTypeValues.map(toLowercaseChoice)}
            bind:values={assayLibraryTypesPred.library_types.contains}
          />
        </Fieldset>
        <Fieldset name="Advanced">
          <div class="collapse">
            <input type="checkbox" />
            <div class="collapse-title font-semibold btn btn-success mb-2">Example</div>
            <div class="collapse-content mockup-code bg-base-200 text-base-content border">
              <pre><code></code></pre>
            </div>
          </div>
          <textarea bind:value={advancedQuery} class="textarea w-full" rows="6"></textarea>
          <p>{advancedQueryError}</p>
        </Fieldset>
        <input name="q" hidden value={stringifiedQuery} />
        <button type="submit" class="btn btn-primary mt-4">Apply</button>
      </form>
    </div>
  </div>
</div>
