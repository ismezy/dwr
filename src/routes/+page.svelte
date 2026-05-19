<script lang="ts">
	import ProjectPanel from '$lib/components/custom/ProjectPanel.svelte';
	import DailyReportPanel from '$lib/components/custom/DailyReportPanel.svelte';
	import ContentPanel from '$lib/components/custom/ContentPanel.svelte';
	import * as Resizable from '$lib/components/ui/resizable/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { i18n } from '$lib/i18n';
	import { reportsStore } from '$lib/stores/reports.svelte';
	import { projectsStore } from '$lib/stores/projects.svelte';
	import { Settings, PanelLeft, PanelLeftClose } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { cn } from '$lib/utils';

	function setMode(mode: 'per-project' | 'summary' | 'all') {
		reportsStore.setMode(mode);
	}

	let showProjectPanel = $state(true);
	let hasAutoCollapsed = $state(false);

	$effect(() => {
		if (hasAutoCollapsed) return;
		const projects = projectsStore.projects;
		if (projectsStore.initialized && projects.length > 0) {
			showProjectPanel = false;
			hasAutoCollapsed = true;
		}
	});
</script>

<div class="flex flex-col h-screen w-full overflow-hidden">
	<!-- Top bar -->
	<div class="flex items-center justify-between px-4 h-12 border-b shrink-0 bg-background">
		<div class="flex items-center gap-2">
			<Button
				variant="ghost"
				size="icon"
				class="h-8 w-8"
				onclick={() => showProjectPanel = !showProjectPanel}
				title={showProjectPanel ? i18n.t('project.collapse') : i18n.t('project.expand')}
			>
				{#if showProjectPanel}
					<PanelLeftClose class="h-4 w-4" />
				{:else}
					<PanelLeft class="h-4 w-4" />
				{/if}
			</Button>
			<span class="text-sm font-semibold">{i18n.t('app.name')}</span>
			<span class="text-xs text-muted-foreground">{i18n.t('app.subtitle')}</span>
		</div>
		<div class="flex items-center gap-3">
			<!-- Mode switcher -->
			<div class="flex rounded-md border bg-background overflow-hidden">
				{#each [{k: 'per-project' as const, l: i18n.t('dailyReport.modePerProject')}, {k: 'summary' as const, l: i18n.t('dailyReport.modeSummary')}, {k: 'all' as const, l: i18n.t('dailyReport.modeAll')}] as item}
					<button
						class={cn(
							'px-3 py-1 text-xs font-medium transition-colors',
							reportsStore.mode === item.k
								? 'bg-accent text-accent-foreground'
								: 'hover:bg-accent/50 text-muted-foreground'
						)}
						onclick={() => setMode(item.k)}
					>
						{item.l}
					</button>
				{/each}
			</div>
			<Button
				variant="ghost"
				size="icon"
				class="h-8 w-8"
				onclick={() => goto('/settings')}
				title={i18n.t('settings.title')}
			>
				<Settings class="h-4 w-4" />
			</Button>
		</div>
	</div>

	<!-- Layout: fixed left sidebar + resizable right two columns -->
	<div class="flex flex-1 overflow-hidden">
		{#if showProjectPanel}
			<ProjectPanel />
		{/if}

		<Resizable.ResizablePaneGroup direction="horizontal" class="flex-1">
			<Resizable.ResizablePane
				defaultSize={30}
				minSize={20}
				maxSize={50}
			>
				<DailyReportPanel />
			</Resizable.ResizablePane>

			<Resizable.ResizableHandle withHandle />

			<Resizable.ResizablePane
				defaultSize={70}
				minSize={30}
			>
				<ContentPanel />
			</Resizable.ResizablePane>
		</Resizable.ResizablePaneGroup>
	</div>
</div>
