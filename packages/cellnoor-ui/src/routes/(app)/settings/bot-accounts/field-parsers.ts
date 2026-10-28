import { onOrAfter, onOrBefore, type Parser } from '#lib/filters/utils.ts';
import type { BotAccountPredicate } from 'cellnoor-client/cellnoor-types.ts';

export enum FormFieldName {
	Description = 'description',
	IsStaff = 'is_staff',
	OwnedBy = 'owned_by',
	CreatedFrom = 'created_from',
	CreatedTo = 'created_to'
}

export const bot_accountFilterFieldParsers: Record<FormFieldName, Parser<BotAccountPredicate>> = {
	description: (v) => ({ description: { trgm_any_unless_empty: v } }),
	is_staff: (v) => ({ is_staff: { in_unless_empty: v.map((s) => s === 'true') } }),
	owned_by: (v) => ({ owned_by: { in_unless_empty: v } }),
	created_from: (v) => onOrAfter(v).map((exp) => ({ created_at: exp })),
	created_to: (v) => onOrBefore(v).map((exp) => ({ created_at: exp }))
};
