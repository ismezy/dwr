<script lang="ts">
	import '../app.css';
	import { configStore } from '$lib/stores/config.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { onMount } from 'svelte';
	import { cn } from '$lib/utils';

	let { children } = $props();

	onMount(() => {
		configStore.refresh();

		const media = window.matchMedia('(prefers-color-scheme: dark)');
		const handler = () => {
			if (configStore.configs.theme === 'system' || !configStore.configs.theme) {
				configStore.applyTheme('system');
			}
		};
		media.addEventListener('change', handler);
		return () => media.removeEventListener('change', handler);
	});
</script>

{@render children()}

<!-- Toast container -->
<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
	{#each toastStore.toasts as toast (toast.id)}
		<div
			class={cn(
				'pointer-events-auto rounded-md px-4 py-2 text-sm font-medium shadow-lg transition-all',
				toast.type === 'success'
					? 'bg-primary text-primary-foreground'
					: 'bg-destructive text-destructive-foreground'
			)}
		>
			{toast.message}
		</div>
	{/each}
</div>
