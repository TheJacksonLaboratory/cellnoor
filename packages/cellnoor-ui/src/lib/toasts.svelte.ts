export const toasts = $state<{ message: string; ok: boolean }[]>([]);

export function toast(message: string, ok: boolean) {
	toasts.push({ message, ok });
	// Every toast lives equally long, so the oldest one is always the next to expire
	setTimeout(() => toasts.shift(), 5000);
}
