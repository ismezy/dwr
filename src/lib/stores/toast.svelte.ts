export interface ToastMessage {
	id: string;
	message: string;
	type: 'success' | 'error';
}

function createToastStore() {
	let toasts = $state<ToastMessage[]>([]);

	function show(message: string, type: 'success' | 'error' = 'success') {
		const id = crypto.randomUUID();
		toasts = [...toasts, { id, message, type }];
		setTimeout(() => {
			toasts = toasts.filter((t) => t.id !== id);
		}, 3000);
	}

	return {
		get toasts() {
			return toasts;
		},
		show,
	};
}

export const toastStore = createToastStore();
