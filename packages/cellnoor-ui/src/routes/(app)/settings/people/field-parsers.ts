import type { Parser } from '#lib/filters/utils.ts';
import type { PersonPredicate } from 'cellnoor-client/cellnoor-types.ts';

export enum FormFieldName {
	Name = 'name',
	Email = 'email',
	IsStaff = 'is_staff',
	InstitutionId = 'institution_id'
}

export const personFilterFieldParsers: Record<FormFieldName, Parser<PersonPredicate>> = {
	name: (v) => ({ name: { trgm_any_unless_empty: v } }),
	email: (v) => ({ email: { like_any_unless_empty: v.map((s) => `%${s}%`) } }),
	is_staff: (v) => ({ is_staff: { in_unless_empty: v.map((s) => s === 'true') } }),
	institution_id: (v) => ({ institution_id: { in_unless_empty: v } })
};
