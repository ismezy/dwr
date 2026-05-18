<script lang="ts">
	import ProjectPanel from '$lib/components/custom/ProjectPanel.svelte';
	import DailyReportPanel from '$lib/components/custom/DailyReportPanel.svelte';
	import ContentPanel from '$lib/components/custom/ContentPanel.svelte';
</script>

<div class="flex h-screen w-full overflow-hidden">
	<ProjectPanel />
	<DailyReportPanel />
	<ContentPanel />
</div>
