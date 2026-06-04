<script lang="ts">
  import InputList from "../../components/InputList.svelte";
  import ChromiumDataset from "./ChromiumDataset.svelte";
  import Fieldset from "../../components/Fieldset.svelte";
  import SortAndLimit from "../../components/SortAndLimit.svelte";
  import {
    type LibraryType,
    type Species,
    type ChromiumDatasetPredicate,
    type ChromiumDatasetPredicateQuery,
    type SpecimenPredicate,
    type TenxAssayPredicate,
    libraryTypeValues,
    type OrderByChromiumDatasetField,
  } from "$lib/cellnoor-types";
  import { isNonempty } from "$lib/query-utils";

  const { data } = $props();
  const { assays, datasets, projects, error } = data;

  let filterForm: HTMLFormElement | undefined = $state();

  const specimenNamePred = $state({ name: { trgm_any: [] } }) satisfies SpecimenPredicate;
  const speciesPred = $state({ species: { in: [] } }) satisfies SpecimenPredicate;
  const tissuePred = $state({ tissue: { trgm_any: [] } }) satisfies SpecimenPredicate;

  const predicates = $derived([
    { specimen: specimenNamePred },
    { specimen: speciesPred },
    { specimen: tissuePred },
  ]) satisfies ChromiumDatasetPredicate[];

  const query = $derived({
    filter: {
      all_of: predicates.filter(isNonempty),
    },
  }) satisfies ChromiumDatasetPredicateQuery;

  const stringifiedQuery = JSON.stringify(query);

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
          bind:orderByField={query.order_by![0]!.field}
          bind:orderByDescending={query.order_by![0]!.descending!}
          choices={orderByChoices}
          parentForm={filterForm!}
        />
      </div>
      {#each chromiumDatasets as cd (cd.id)}
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
            bind:values={query.filter.specimen.names}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Species"
            choices={speciesChoices}
            bind:values={query.filter.specimen.species}
          />
        </Fieldset>
        <Fieldset name="Lab Information">
          <InputList
            parentForm={filterForm}
            fieldName="Lab Name"
            choices={(projects || []).map((p) => {
              return { label: p.name, value: p.id };
            })}
            bind:values={query.filter.project_ids}
          />
        </Fieldset>
        <Fieldset name="Assay Information">
          <InputList
            parentForm={filterForm}
            fieldName="Assay Name"
            choices={assays!.map((a) => {
              return { label: a.name, value: a.name };
            })}
            bind:values={query.filter.assay.names}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Multiplexing"
            choices={plexyChoices}
            bind:values={query.filter.assay.sample_multiplexing}
          />
          <InputList
            parentForm={filterForm}
            fieldName="Library Types"
            choices={libraryTypeChoices}
            bind:values={query.filter.assay.library_types_flat}
          />
        </Fieldset>
        <Fieldset name="Advanced">
          <div class="collapse">
            <input type="checkbox" />
            <div class="collapse-title font-semibold btn btn-success mb-2">Example</div>
            <div class="collapse-content mockup-code bg-base-200 text-base-content border">
              <pre><code>{exampleAdvancedQuery}</code></pre>
            </div>
          </div>
          <textarea
            bind:value={advancedQuery}
            class="textarea w-full"
            rows="6"
            placeholder={advancedQueryPlaceholder}
          ></textarea>
          <p>{advancedQueryError}</p>
        </Fieldset>
        <input name="q" hidden bind:value={stringifiedQuery} />
        <button type="submit" class="btn btn-primary mt-4">Apply</button>
      </form>
    </div>
  </div>
</div>
