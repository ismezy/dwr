<script lang="ts">
	import { goto } from '$app/navigation';
	import { projectsStore } from '$lib/stores/projects.svelte';
	import { configStore } from '$lib/stores/config.svelte';
	import { reportsStore } from '$lib/stores/reports.svelte';
	import { i18n } from '$lib/i18n';
	import { Button } from '$lib/components/ui/button/index.js';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { CalendarDays, FileText, Sparkles } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	let showAiAlert = $state(false);

	function getTodayStr() {
		const d = new Date();
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
	}

	function isAiConfigured(): boolean {
		const cfg = configStore.configs;
		return !!cfg.ai_provider && !!cfg.ai_api_key;
	}

	async function handleGenerate() {
		const project = projectsStore.selected;
		if (!project) return;

		if (!isAiConfigured()) {
			showAiAlert = true;
			return;
		}

		const gitUser = projectsStore.resolveGitUserName(project);
		await reportsStore.generateReport(
			project.path,
			project.name,
			gitUser,
			getTodayStr(),
			configStore.configs.work_dir
		);
	}

	function goToSettings() {
		showAiAlert = false;
		goto('/settings');
	}

	async function handleSelect(date: string) {
		const project = projectsStore.selected;
		if (!project) return;
		await reportsStore.readReport(
			project.path,
			project.name,
			date,
			configStore.configs.work_dir
		);
	}

	$effect(() => {
		const project = projectsStore.selected;
		if (project) {
			reportsStore.loadReports(project.path, project.name, configStore.configs.work_dir);
			reportsStore.selectDate(null);
		}
	});
</script>

<div class="flex flex-col h-full w-full bg-muted/30">
	<div class="flex items-center justify-between px-3 py-3 border-b shrink-0 gap-2">
		<div class="flex items-center gap-2 min-w-0">
			<CalendarDays class="h-4 w-4 text-muted-foreground shrink-0" />
			<span class="text-sm font-semibold truncate">{i18n.t('dailyReport.title')}</span>
		</div>
		{#if projectsStore.selected}
			<Button
				variant="ghost"
				size="sm"
				class="h-7 px-2 text-xs shrink-0"
				onclick={handleGenerate}
				disabled={reportsStore.generating}
				title={i18n.t('dailyReport.generate')}
			>
				<Sparkles class="h-3 w-3 mr-1" />
				{reportsStore.generating ? i18n.t('dailyReport.generating') : i18n.t('dailyReport.generate')}
			</Button>
		{/if}
	</div>

	{#if projectsStore.selected}
		<ScrollArea class="flex-1">
			<div class="p-2 space-y-1">
				{#each reportsStore.reports as report (report.date)}
					<button
						class={cn(
							'w-full text-left rounded-md px-3 py-2 transition-colors flex items-center gap-2',
							reportsStore.selectedDate === report.date
								? 'bg-accent text-accent-foreground'
								: 'hover:bg-accent/50 text-foreground'
						)}
						onclick={() => handleSelect(report.date)}
					>
						<FileText class="h-4 w-4 shrink-0 opacity-60" />
						<div class="flex-1 min-w-0">
							<div class="text-sm font-medium">{report.date}</div>
						</div>
					</button>
				{:else}
					<div class="text-xs text-muted-foreground text-center py-8 px-2">
						{i18n.t('dailyReport.emptyHint')}
					</div>
				{/each}
			</div>
		</ScrollArea>
	{:else}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-sm text-muted-foreground text-center px-4">
				{i18n.t('dailyReport.emptyHint')}
			</div>
		</div>
	{/if}
</div>

<AlertDialog.Root bind:open={showAiAlert}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{i18n.t('dailyReport.aiConfigTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				{i18n.t('dailyReport.aiNotConfigured')}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel onclick={() => (showAiAlert = false)}>
				{i18n.t('common.cancel')}
			</AlertDialog.Cancel>
			<AlertDialog.Action onclick={goToSettings}>
				{i18n.t('settings.title')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
