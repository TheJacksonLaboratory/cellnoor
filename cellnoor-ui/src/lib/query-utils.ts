import type { SpecimenFilter, TenxAssayFilter } from "cellnoor-client";
import { expect, test } from "bun:test";

const whitespaceRegex = /\s|_|-/g;

function subWhitespaceWithPercent(s: unknown) {
  if (typeof s !== "string") {
    return s;
  }

  return `%${s.replace(whitespaceRegex, "_")}%`;
}

function jsonReplacer(_key: string | number, value: unknown) {
  if (!Array.isArray(value)) {
    return value;
  }

  if (value.length === 0) {
    return undefined;
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

export class Query<F, O> {
  filter: Filter<F>;
  limit: number;
  offset?: number;
  order_by: O[];

  constructor(filter: Filter<F>, limit: number = 500, orderBy: O[] = []) {
    this.filter = filter;
    this.limit = limit;
    this.order_by = orderBy;
  }

  toQuerystring() {
    return JSON.stringify(this, jsonReplacer);
  }
}

export const emptyAssayFilter: Filter<TenxAssayFilter> = {
  ids: [],
  names: [],
  library_types: [],
  chromium_chips: [],
  chemistry_versions: [],
  sample_multiplexing: [],
};

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
  additional_data: {},
  submitted_by: [],
  returned_by: [],
  types: [],
};
