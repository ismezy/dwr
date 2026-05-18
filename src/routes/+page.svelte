<script lang="ts">
	import ProjectPanel from '$lib/components/custom/ProjectPanel.svelte';
	import DailyReportPanel from '$lib/components/custom/DailyReportPanel.svelte';
	import ContentPanel from '$lib/components/custom/ContentPanel.svelte';
	import ConfigDialog from '$lib/components/custom/ConfigDialog.svelte';
</script>

<div class="flex flex-col h-screen w-full overflow-hidden">
	<!-- Top bar -->
	<div class="flex items-center justify-between px-4 h-12 border-b shrink-0 bg-background">
		<div class="flex items-center gap-2">
			<span class="text-sm font-semibold">DWR</span>
			<span class="text-xs text-muted-foreground">日报生成器</span>
		</div>
		<ConfigDialog />
	</div>

	<!-- Three columns -->
	<div class="flex flex-1 overflow-hidden">
		<ProjectPanel />
		<DailyReportPanel />
		<ContentPanel />
	</div>
</div>
