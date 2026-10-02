import { afterNavigate } from '$app/navigation';

export class Selection {
	checked: Record<string, boolean> = $state({});
	count = $derived(Object.values(this.checked).filter(Boolean).length);

	constructor() {
		afterNavigate(() => (this.checked = {}));
	}

	allChecked(ids: string[]) {
		return ids.length > 0 && ids.every((id) => this.checked[id]);
	}

	toggleAll(ids: string[]) {
		this.checked = this.allChecked(ids) ? {} : Object.fromEntries(ids.map((id) => [id, true]));
	}
}
