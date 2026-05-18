<script lang="ts">
	import '../app.css';
	import { configStore } from '$lib/stores/config.svelte';
	import { onMount } from 'svelte';

	let { children } = $props();

	onMount(() => {
		// 初始化配置（含主题）
		configStore.refresh();

		// 监听系统主题变化
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
