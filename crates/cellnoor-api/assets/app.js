document.addEventListener('change', ({ target }) => {
	if (target.matches('[data-autosubmit]')) {
		target.form.requestSubmit();
	}
});

// Without this script, a dropdown's search box only filters its options when the form is submitted
document.addEventListener('input', ({ target }) => {
	const menu = target.closest('.combobox .menu');
	if (!menu || target.type !== 'search') return;

	const search = target.value.toLowerCase();
	const items = [...menu.querySelectorAll('.menu-item')];

	for (const item of items) {
		item.hidden = !item.dataset.label.toLowerCase().includes(search);
	}
	menu.querySelector('.empty').hidden = items.some((item) => !item.hidden);
});

document.addEventListener('keydown', (event) => {
	const menu = event.target.closest('.combobox .menu');
	if (event.key !== 'Enter' || !menu || event.target.type !== 'search') return;

	event.preventDefault();
	menu.querySelector('.menu-item:not([hidden])')?.click();
});

document.querySelector('[data-back]')?.addEventListener('click', () => history.back());

document.querySelector('[data-autofocus]')?.focus({ preventScroll: true });

// A page restored from the back/forward cache shows what the user did before leaving it, like a
// checkbox whose change the server rejected, rather than what the server has
addEventListener('pageshow', (event) => {
	if (event.persisted) location.reload();
});

// Filters, "Load more", and mutations all reload the page they were submitted from, so that page keeps
// its scroll positions
const scrollKey = `scroll:${location.pathname}`;
const sidebar = document.querySelector('aside');

addEventListener('pagehide', () => {
	sessionStorage.setItem(scrollKey, JSON.stringify([scrollY, sidebar?.scrollTop ?? 0]));
});

const [navigation] = performance.getEntriesByType('navigation');
const fromSamePage = document.referrer && new URL(document.referrer).pathname === location.pathname;

if (navigation?.type === 'navigate' && fromSamePage) {
	const [pageY, sidebarY] = JSON.parse(sessionStorage.getItem(scrollKey) ?? '[0, 0]');
	scrollTo(0, pageY);
	sidebar?.scrollTo(0, sidebarY);
}
