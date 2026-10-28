import { onOrAfter, onOrBefore, type Parser } from '#lib/filters/utils.ts';
import type { ProjectPredicate } from 'cellnoor-client/cellnoor-types.ts';

export enum FormFieldName {
	Name = 'name',
	CreatedBy = 'created_by',
	ActiveFrom = 'active_from',
	ActiveTo = 'active_to'
}

export const projectFilterFieldParsers: Record<FormFieldName, Parser<ProjectPredicate>> = {
	name: (v) => ({ name: { trgm_any_unless_empty: v } }),
	created_by: (v) => ({ created_by: { in_unless_empty: v } }),
	active_from: (v) => onOrAfter(v).map((exp) => ({ started_at: exp })),
	active_to: (v) => onOrBefore(v).map((exp) => ({ ended_at: exp }))
};
