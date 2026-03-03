import type {
  LibraryType,
  SpecimenFilter,
  TenxAssayFilter,
} from "cellnoor-client";

const whitespaceRegex = /\s|_|-/g;

function subWhitespaceWithPercent(s: unknown) {
  if (typeof s !== "string") {
    return s;
  }

  return `%${s.replace(whitespaceRegex, "_")}%`;
}

const DO_NOT_REPLACE = [
  "species",
  "host_species",
  "library_types",
  "type",
  "project_ids",
  "sample_multiplexing",
];

const libraryTypes: LibraryType[] = [
  "antibody_capture",
  "antigen_capture",
  "chromatin_accessibility",
  "crispr_guide_capture",
  "custom",
  "gene_expression",
  "multiplexing_capture",
  "vdj",
  "vdj_b",
  "vdj_t",
  "vdj_t_gd",
];

function jsonReplacer(key: string | number, value: unknown) {
  if (!Array.isArray(value)) {
    return value;
  }

  if (value.length === 0) {
    return undefined;
  }

  // This is a bit of a hack because the backend expects a very rigid set of enumerated items for some keys
  if (typeof key === "string" && DO_NOT_REPLACE.includes(key)) {
    return value;
  } else if (libraryTypes.includes(value[0])) {
    return value;
  }

  return value.map(subWhitespaceWithPercent);
}

type Primitive = boolean | number | bigint | string;

type PrimitiveFields<T> = {
  [K in keyof T as NonNullable<T[K]> extends Primitive ? K : never]: T[K];
};

type ObjectFields<T> = {
  [K in keyof T as NonNullable<T[K]> extends object ? K : never]-?: Filter<
    NonNullable<T[K]>
  >;
};

type ArrayFields<T> = {
  [
    K in keyof T as NonNullable<T[K]> extends (infer _)[] ? K
      : never
  ]-?: NonNullable<T[K]>;
};

type Filter<T> = PrimitiveFields<T> & ObjectFields<T> & ArrayFields<T>;

export interface Query<F, O> {
  filter: Filter<F>;
  limit: number;
  offset?: number;
  order_by?: O[];
}

export function toQueryString<F, O>(q: Query<F, O>) {
  return JSON.stringify(q, jsonReplacer);
}

export const emptyAssayFilter: Filter<TenxAssayFilter> = {
  ids: [],
  names: [],
  library_types: [],
  library_types_flat: [],
  chromium_chips: [],
  chemistry_versions: [],
  sample_multiplexing: [],
};

// @ts-expect-error
export const emptySpecimenFilter: Filter<SpecimenFilter> = {
  ids: [],
  names: [],
  species: [],
  host_species: [],
  tissues: [],
  thermal_preservation_methods: [],
  fixatives: [],
  embedded_in: [],
  project_ids: [],
  submitted_by: [],
  returned_by: [],
  types: [],
};
