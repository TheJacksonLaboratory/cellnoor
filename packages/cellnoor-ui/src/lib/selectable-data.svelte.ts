import { SvelteMap } from 'svelte/reactivity';

export class SelectableData<T> {
	#items: SvelteMap<string, { item: T; selected: boolean }>;
	#size: number;
	#nSelected: number;
	#allSelected: boolean;

	// This takes a closure so we don't have to iterate
	constructor(items: T[], getId: (i: T) => string) {
		this.#items = new SvelteMap(items.map((item) => [getId(item), { item, selected: false }]));

		this.#size = $derived(this.#items.size);

		// This looks convoluted, but it's better for performance
		this.#nSelected = $derived(this.#items.values().reduce((n, { selected }) => n + +selected, 0));

		this.#allSelected = $derived(this.nSelected ? this.nSelected === this.size : false);
	}

	get size() {
		return this.#size;
	}

	get nSelected() {
		return this.#nSelected;
	}

	get allSelected() {
		return this.#allSelected;
	}

	get(id: string) {
		return this.#items.get(id);
	}

	entries() {
		return this.#items.entries();
	}

	toggleAll() {
		const currentState = this.allSelected;

		for (const [id, { item }] of this.#items.entries()) {
			this.#items.set(id, { item, selected: !currentState });
		}
	}
}
