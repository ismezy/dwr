<script lang="ts">
	import { goto } from '$app/navigation';
	import { projectsStore } from '$lib/stores/projects.svelte';
	import { configStore } from '$lib/stores/config.svelte';
	import { reportsStore, getWeekRange } from '$lib/stores/reports.svelte';
	import { i18n } from '$lib/i18n';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { FileText, User, Folder, Hash, RefreshCw, Loader, Eye, PenLine, Save, Wand2 } from '@lucide/svelte';
	import { marked } from 'marked';
	import { cn } from '$lib/utils';

	let showAiAlert = $state(false);
	let viewMode = $state<'preview' | 'edit'>('preview');
	let editContent = $state('');
	let saving = $state(false);
	let polishing = $state(false);

	function isAiConfigured(): boolean {
		const cfg = configStore.configs;
		return !!cfg.ai_provider && !!cfg.ai_api_key;
	}

	function currentDate() {
		return reportsStore.summarySelectedDate ?? reportsStore.selectedDate;
	}

	function currentContent() {
		return reportsStore.summarySelectedDate ? reportsStore.summaryContent : reportsStore.content;
	}

	function isSummaryActive() {
		return !!reportsStore.summarySelectedDate;
	}

	function renderedMarkdown() {
		const content = currentContent();
		if (!content) return '';
		return marked.parse(content, { async: false }) as string;
	}

	function startEdit() {
		editContent = currentContent();
		viewMode = 'edit';
	}

	function cancelEdit() {
		viewMode = 'preview';
		editContent = '';
	}

	async function handleSave() {
		if (!currentDate()) return;
		saving = true;
		try {
			if (isSummaryActive()) {
				await reportsStore.saveSummaryReport(
					reportsStore.summarySelectedDate!,
					editContent,
					configStore.configs.work_dir
				);
				reportsStore.summaryContent = editContent;
			} else {
				const project = projectsStore.selected;
				if (!project) return;
				await reportsStore.saveReport(
					project.path,
					project.name,
					reportsStore.selectedDate!,
					editContent,
					configStore.configs.work_dir
				);
				reportsStore.content = editContent;
			}
			viewMode = 'preview';
		} catch (e) {
			console.error('failed to save report:', e);
		} finally {
			saving = false;
		}
	}

	async function handlePolish() {
		if (!editContent.trim()) return;
		polishing = true;
		try {
			const polished = await reportsStore.polish(editContent);
			editContent = polished;
		} catch (e) {
			console.error('failed to polish report:', e);
		} finally {
			polishing = false;
		}
	}

	async function handleRegenerate() {
		const project = projectsStore.selected;
		if (!project) return;
		if (!currentDate()) return;

		if (!isAiConfigured()) {
			showAiAlert = true;
			return;
		}

		let weekEnd: string | undefined;
		if (reportsStore.reportPeriod === 'weekly') {
			const range = getWeekRange(currentDate()!, configStore.configs.week_start_day ?? 1);
			weekEnd = range.end;
		}

		if (isSummaryActive()) {
			await reportsStore.generateSummaryReport(
				reportsStore.summarySelectedDate!,
				configStore.configs.work_dir,
				weekEnd
			);
		} else {
			const gitUser = projectsStore.resolveGitUserName(project);
			await reportsStore.generateReport(
				project.path,
				project.name,
				gitUser,
				reportsStore.selectedDate!,
				configStore.configs.work_dir,
				weekEnd
			);
		}
		viewMode = 'preview';
	}

	function goToSettings() {
		showAiAlert = false;
		goto('/settings');
	}
</script>

<div class="flex flex-col h-full w-full bg-background">
	<div class="flex items-center gap-2 px-4 py-3 border-b shrink-0">
		<FileText class="h-4 w-4 text-muted-foreground" />
		<span class="text-sm font-semibold">{i18n.t('content.title')}</span>
	</div>

	{#if currentDate()}
		<!-- 固定标题栏 -->
		<div class="flex items-center justify-between px-6 pt-4 pb-2 shrink-0 border-b">
			<div class="text-2xl font-bold">
				{#if isSummaryActive()}
					{reportsStore.reportPeriod === 'weekly' ? i18n.t('dailyReport.summaryTitleWeekly') : i18n.t('dailyReport.summaryTitle')}
				{:else}
					{#if i18n.locale === 'en'}{projectsStore.selected?.name ?? ''} {reportsStore.reportPeriod === 'weekly' ? i18n.t('content.weeklyReport') : i18n.t('content.dailyReport')}{:else}{projectsStore.selected?.name ?? ''}{reportsStore.reportPeriod === 'weekly' ? i18n.t('content.weeklyReport') : i18n.t('content.dailyReport')}{/if}
				{/if}
			</div>
			<div class="flex items-center gap-2">
				{#if viewMode === 'edit'}
					<Button
						variant="ghost"
						size="sm"
						class="h-8 px-2 text-xs"
						onclick={cancelEdit}
						disabled={saving || polishing}
					>
						{i18n.t('common.cancel')}
					</Button>
					<Button
						variant="secondary"
						size="sm"
						class="h-8 px-2 text-xs"
						onclick={handlePolish}
						disabled={saving || polishing}
					>
						{#if polishing}
							<Loader class="h-3 w-3 mr-1 animate-spin" />
						{:else}
							<Wand2 class="h-3 w-3 mr-1" />
						{/if}
						{polishing ? i18n.t('content.polishing') : i18n.t('content.polish')}
					</Button>
					<Button
						variant="default"
						size="sm"
						class="h-8 px-2 text-xs"
						onclick={handleSave}
						disabled={saving || polishing}
					>
						<Save class="h-3 w-3 mr-1" />
						{saving ? i18n.t('common.saving') : i18n.t('common.save')}
					</Button>
				{:else}
					<div class="flex rounded-md border bg-background overflow-hidden">
						<button
							class={cn(
								'px-2 py-1.5 text-xs font-medium transition-colors flex items-center gap-1',
								viewMode === 'preview'
									? 'bg-accent text-accent-foreground'
									: 'hover:bg-accent/50 text-muted-foreground'
							)}
							onclick={() => (viewMode = 'preview')}
						>
							<Eye class="h-3 w-3" />
							{i18n.t('content.preview')}
						</button>
						<button
							class={cn(
								'px-2 py-1.5 text-xs font-medium transition-colors flex items-center gap-1',
								viewMode === 'edit'
									? 'bg-accent text-accent-foreground'
									: 'hover:bg-accent/50 text-muted-foreground'
							)}
							onclick={startEdit}
						>
							<PenLine class="h-3 w-3" />
							{i18n.t('content.edit')}
						</button>
					</div>
					<Button
						variant="outline"
						size="sm"
						class="h-8 px-2 text-xs"
						onclick={handleRegenerate}
						disabled={reportsStore.generating || reportsStore.summaryGenerating}
						title={i18n.t('content.regenerate')}
					>
						{#if reportsStore.generating || reportsStore.summaryGenerating}
							<Loader class="h-3 w-3 mr-1 animate-spin" />
						{:else}
							<RefreshCw class="h-3 w-3 mr-1" />
						{/if}
						{reportsStore.generating || reportsStore.summaryGenerating ? i18n.t('dailyReport.generating') : i18n.t('content.regenerate')}
					</Button>
				{/if}
			</div>
		</div>

		<!-- 滚动内容区 -->
		<div class="flex-1 overflow-y-auto relative px-6 pb-6">
			{#if reportsStore.generating || reportsStore.summaryGenerating}
				<div class="sticky top-0 inset-x-0 h-0 flex justify-center z-10 pointer-events-none">
					<div class="mt-4 inline-flex items-center gap-2 px-3 py-2 rounded-md bg-background/90 border shadow-sm pointer-events-auto">
						<Loader class="h-4 w-4 animate-spin text-primary" />
						<span class="text-sm text-muted-foreground">{i18n.t('dailyReport.generating')}</span>
					</div>
				</div>
			{/if}
			{#if currentContent()}
				{#if viewMode === 'edit'}
					<Textarea
						bind:value={editContent}
						class="w-full min-h-[400px] font-mono text-sm leading-relaxed mt-4"
					/>
				{:else}
					<div class="markdown-body max-w-none mt-2">
						{@html renderedMarkdown()}
					</div>
				{/if}
			{:else}
				<div class="rounded-lg border bg-card p-6 mt-4">
					<div class="text-sm text-muted-foreground text-center">
						{reportsStore.reportPeriod === 'weekly' ? i18n.t('content.placeholderWeekly') : i18n.t('content.placeholder')}
					</div>
				</div>
			{/if}
		</div>
	{:else if projectsStore.selected}
		<div class="flex-1 overflow-y-auto relative px-6 pb-6">
			<div class="text-2xl font-bold mb-2 mt-4">{projectsStore.selected.name}</div>

			<div class="space-y-3 text-sm text-muted-foreground mb-6">
				{#if projectsStore.selected.code}
					<div class="flex items-center gap-2">
						<Hash class="h-4 w-4" />
						<span>{projectsStore.selected.code}</span>
					</div>
				{/if}
				<div class="flex items-center gap-2">
					<Folder class="h-4 w-4" />
					<span class="truncate">{projectsStore.selected.path}</span>
				</div>
				<div class="flex items-center gap-2">
					<User class="h-4 w-4" />
					<span>
						{i18n.t('config.gitUserName')}:
						{#if projectsStore.selected.git_user_name}
							{projectsStore.selected.git_user_name}
							<span class="text-xs text-muted-foreground/60">({i18n.t('content.gitUser.project')})</span>
						{:else if configStore.configs.git_user_name}
							{configStore.configs.git_user_name}
							<span class="text-xs text-muted-foreground/60">({i18n.t('content.gitUser.global')})</span>
						{:else}
							<span class="text-xs text-destructive">{i18n.t('content.gitUser.unconfigured')}</span>
						{/if}
					</span>
				</div>
			</div>

			<div class="rounded-lg border bg-card p-6">
				<div class="text-sm text-muted-foreground text-center">
					{reportsStore.reportPeriod === 'weekly' ? i18n.t('content.placeholderWeekly') : i18n.t('content.placeholder')}
				</div>
			</div>
		</div>
	{:else}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-sm text-muted-foreground text-center px-4">
				{i18n.t('content.emptyHint')}
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
