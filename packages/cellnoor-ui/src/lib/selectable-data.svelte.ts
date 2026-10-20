import { SvelteMap } from 'svelte/reactivity';

export class SelectableData<T> {
	#items: SvelteMap<string, { item: T; selected: boolean }>;
	#nSelected: number;
	#allItemsAreSelected: boolean;

	// This takes a closure rather than a list of IDs so that the caller doesn't have to iterate over the list just for this constructor to iterate over the list again
	constructor(items: T[], getId: (i: T) => string) {
		this.#items = new SvelteMap(items.map((item) => [getId(item), { item, selected: false }]));

		// This looks convoluted, but it's better for performance
		this.#nSelected = $derived(this.#items.values().reduce((n, { selected }) => n + +selected, 0));

		this.#allItemsAreSelected = $derived(this.nSelected > 0 && this.nSelected === this.size);
	}

	get size() {
		return this.#items.size;
	}

	get nSelected() {
		return this.#nSelected;
	}

	get allSelected() {
		return this.#allItemsAreSelected;
	}

	get(id: string) {
		return this.#items.get(id)?.item;
	}

	entries() {
		return this.#items.entries().map(([id, { item }]) => [id, item] as const);
	}

	isSelectedGetter(id: string) {
		return () => this.#items.get(id)!.selected;
	}

	isSelectedSetter(id: string) {
		const { item } = this.#items.get(id)!;

		return (selected: boolean) => this.#items.set(id, { item, selected });
	}

	toggleAll() {
		const currentState = this.allSelected;

		for (const [id, { item }] of this.#items.entries()) {
			this.#items.set(id, { item, selected: !currentState });
		}
	}

	get selectedItems() {
		return this.#items
			.values()
			.filter(({ selected }) => selected)
			.map(({ item }) => item);
	}
}
