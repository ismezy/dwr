<script lang="ts">
	import { goto } from '$app/navigation';
	import { projectsStore } from '$lib/stores/projects.svelte';
	import { configStore } from '$lib/stores/config.svelte';
	import { reportsStore, getWeekRange } from '$lib/stores/reports.svelte';
	import { i18n } from '$lib/i18n';
	import { Button } from '$lib/components/ui/button/index.js';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { CalendarDays, FileText, Sparkles, Loader, ChevronDown } from '@lucide/svelte';
	import { cn } from '$lib/utils';
	import * as Calendar from '$lib/components/ui/calendar/index.js';
	import { CalendarDate, type DateValue } from '@internationalized/date';

	let showAiAlert = $state(false);
	let showGenerateMenu = $state(false);
	let showDateDialog = $state(false);
	let customCalendarValue = $state<DateValue | undefined>(undefined);

	function calendarValueToStr(value: DateValue | undefined): string {
		if (!value) return '';
		return `${value.year}-${String(value.month).padStart(2, '0')}-${String(value.day).padStart(2, '0')}`;
	}

	function getTodayStr() {
		const d = new Date();
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
	}

	function offsetDate(dateStr: string, days: number): string {
		const date = new Date(dateStr + 'T00:00:00');
		date.setDate(date.getDate() + days);
		return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
	}

	function getDailyOptions() {
		const today = getTodayStr();
		return [
			{ label: i18n.t('dailyReport.yesterday'), date: offsetDate(today, -1) },
			{ label: i18n.t('dailyReport.dayBeforeYesterday'), date: offsetDate(today, -2) },
			{ label: i18n.t('dailyReport.threeDaysAgo'), date: offsetDate(today, -3) },
		];
	}

	function getWeeklyOptions() {
		const today = getTodayStr();
		const thisWeek = getWeekRange(today, configStore.configs.week_start_day ?? 1);
		return [
			{
				label: i18n.t('dailyReport.lastWeek'),
				start: offsetDate(thisWeek.start, -7),
				end: offsetDate(thisWeek.end, -7),
			},
			{
				label: i18n.t('dailyReport.twoWeeksAgo'),
				start: offsetDate(thisWeek.start, -14),
				end: offsetDate(thisWeek.end, -14),
			},
		];
	}

	function initCustomDate() {
		const d = new Date();
		d.setDate(d.getDate() - 4);
		customCalendarValue = new CalendarDate(d.getFullYear(), d.getMonth() + 1, d.getDate());
	}

	$effect(() => {
		// Reset custom date when period changes
		const _ = reportsStore.reportPeriod;
		initCustomDate();
	});

	function isAiConfigured(): boolean {
		const cfg = configStore.configs;
		return !!cfg.ai_provider && !!cfg.ai_api_key;
	}

	async function handleGenerateProject(date: string, weekEnd?: string) {
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
			date,
			configStore.configs.work_dir,
			weekEnd
		);
	}

	async function handleGenerateSummary(date: string, weekEnd?: string) {
		if (!isAiConfigured()) {
			showAiAlert = true;
			return;
		}
		await reportsStore.generateSummaryReport(
			date,
			configStore.configs.work_dir,
			weekEnd
		);
	}

	async function handleGenerateAll(date: string, weekEnd?: string) {
		if (!isAiConfigured()) {
			showAiAlert = true;
			return;
		}
		const project = projectsStore.selected;
		if (project) {
			const gitUser = projectsStore.resolveGitUserName(project);
			await reportsStore.generateReport(
				project.path,
				project.name,
				gitUser,
				date,
				configStore.configs.work_dir,
				weekEnd
			);
		}
		await reportsStore.generateSummaryReport(
			date,
			configStore.configs.work_dir,
			weekEnd
		);
	}

	function handleGenerate(date: string, weekEnd?: string) {
		const mode = reportsStore.mode;
		if (mode === 'per-project') {
			handleGenerateProject(date, weekEnd);
		} else if (mode === 'summary') {
			handleGenerateSummary(date, weekEnd);
		} else {
			handleGenerateAll(date, weekEnd);
		}
	}

	function handleGenerateToday() {
		if (reportsStore.reportPeriod === 'weekly') {
			const { start, end } = getWeekRange(getTodayStr(), configStore.configs.week_start_day ?? 1);
			handleGenerate(start, end);
		} else {
			handleGenerate(getTodayStr());
		}
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

	async function handleSelectSummary(date: string) {
		await reportsStore.readSummaryReport(date, configStore.configs.work_dir);
	}

	function setReportPeriod(period: 'daily' | 'weekly') {
		reportsStore.setReportPeriod(period);
	}

	function closeMenu() {
		showGenerateMenu = false;
	}

	function openDateDialog() {
		closeMenu();
		showDateDialog = true;
	}

	function confirmCustomDate() {
		showDateDialog = false;
		handleGenerate(calendarValueToStr(customCalendarValue));
	}

	$effect(() => {
		if (!showGenerateMenu) return;
		function handleDocClick() {
			showGenerateMenu = false;
		}
		// Delay to avoid immediate close from the same click that opened
		setTimeout(() => {
			document.addEventListener('click', handleDocClick, { once: true });
		}, 10);
		return () => {
			document.removeEventListener('click', handleDocClick);
		};
	});

	$effect(() => {
		const project = projectsStore.selected;
		if (project) {
			reportsStore.loadReports(project.path, project.name, configStore.configs.work_dir);
			reportsStore.selectDate(null);
		}
	});

	$effect(() => {
		if (reportsStore.mode === 'summary' || reportsStore.mode === 'all') {
			reportsStore.loadSummaryReports(configStore.configs.work_dir);
		}
	});

	$effect(() => {
		// Reload when report period changes
		const period = reportsStore.reportPeriod;
		const project = projectsStore.selected;
		if (project) {
			reportsStore.loadReports(project.path, project.name, configStore.configs.work_dir);
		}
		if (reportsStore.mode === 'summary' || reportsStore.mode === 'all') {
			reportsStore.loadSummaryReports(configStore.configs.work_dir);
		}
	});
</script>

<div class="flex flex-col h-full w-full bg-muted/30">
	<div class="flex items-center justify-between px-3 py-3 border-b shrink-0 gap-2">
		<div class="flex items-center gap-2 min-w-0">
			<CalendarDays class="h-4 w-4 text-muted-foreground shrink-0" />
			<!-- Report period switcher -->
			<div class="flex rounded-md border bg-background overflow-hidden">
				{#each [{k: 'daily' as const, l: i18n.t('dailyReport.daily')}, {k: 'weekly' as const, l: i18n.t('dailyReport.weekly')}] as item}
					<button
						class={cn(
							'px-2 py-1 text-xs font-medium transition-colors',
							reportsStore.reportPeriod === item.k
								? 'bg-accent text-accent-foreground'
								: 'hover:bg-accent/50 text-muted-foreground'
						)}
						onclick={() => setReportPeriod(item.k)}
					>
						{item.l}
					</button>
				{/each}
			</div>
		</div>
		{#if projectsStore.selected}
			<div class="flex items-center">
				<Button
					variant="ghost"
					size="sm"
					class="h-7 px-2 text-xs shrink-0 rounded-r-none"
					onclick={handleGenerateToday}
					disabled={reportsStore.generating || reportsStore.summaryGenerating}
				>
					{#if reportsStore.generating || reportsStore.summaryGenerating}
						<Loader class="h-3 w-3 mr-1 animate-spin" />
					{:else}
						<Sparkles class="h-3 w-3 mr-1" />
					{/if}
					{reportsStore.generating || reportsStore.summaryGenerating ? i18n.t('dailyReport.generating') : (reportsStore.reportPeriod === 'weekly' ? i18n.t('dailyReport.generateWeekly') : i18n.t('dailyReport.generate'))}
				</Button>
				<div class="relative">
					<Button
						variant="ghost"
						size="icon"
						class="h-7 w-6 rounded-l-none border-l border-border/50"
						onclick={(e) => { e.stopPropagation(); showGenerateMenu = !showGenerateMenu; }}
						disabled={reportsStore.generating || reportsStore.summaryGenerating}
					>
						<ChevronDown class="h-3 w-3" />
					</Button>
					{#if showGenerateMenu}
						<div class="generate-menu absolute right-0 top-full mt-1 w-52 rounded-md border bg-popover shadow-md z-50 py-1">
							{#if reportsStore.reportPeriod === 'daily'}
								{#each getDailyOptions() as opt}
									<button
										class="w-full text-left px-3 py-1.5 text-sm hover:bg-accent transition-colors"
										onclick={() => { closeMenu(); handleGenerate(opt.date); }}
									>
										{i18n.t('dailyReport.generate').replace('今天', opt.label)}
									</button>
								{/each}
								<div class="border-t my-1"></div>
								<button
									class="w-full text-left px-3 py-1.5 text-sm hover:bg-accent transition-colors"
									onclick={openDateDialog}
								>
									{i18n.t('dailyReport.selectDate')}
								</button>
							{:else}
								{#each getWeeklyOptions() as opt}
									<button
										class="w-full text-left px-3 py-1.5 text-sm hover:bg-accent transition-colors"
										onclick={() => { closeMenu(); handleGenerate(opt.start, opt.end); }}
									>
										{i18n.t('dailyReport.generateWeekly').replace('本周', opt.label)}
									</button>
								{/each}
							{/if}
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>

	{#if projectsStore.selected || reportsStore.mode === 'summary'}
		{@const isGenerating = reportsStore.generating || reportsStore.summaryGenerating}
		<ScrollArea class="flex-1 {isGenerating ? 'opacity-50 pointer-events-none' : ''}">
			<div class="p-2 space-y-4">
				<!-- Summary section (visible in summary / all modes) -->
				{#if reportsStore.mode === 'summary' || reportsStore.mode === 'all'}
					<div>
						<div class="px-2 py-1 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
							{reportsStore.reportPeriod === 'weekly' ? i18n.t('dailyReport.summarySectionWeekly') : i18n.t('dailyReport.summarySection')}
						</div>
						<div class="space-y-1 mt-1">
							{#each reportsStore.summaryReports as report (report.date)}
								<button
									class={cn(
										'w-full text-left rounded-md px-3 py-2 transition-colors flex items-center gap-2',
										reportsStore.summarySelectedDate === report.date
											? 'bg-accent text-accent-foreground'
											: 'hover:bg-accent/50 text-foreground'
									)}
									onclick={() => handleSelectSummary(report.date)}
								>
									<FileText class="h-4 w-4 shrink-0 opacity-60" />
									<div class="flex-1 min-w-0">
										<div class="text-sm font-medium">{report.date}</div>
									</div>
								</button>
							{:else}
								<div class="text-xs text-muted-foreground text-center py-4 px-2">
									{reportsStore.reportPeriod === 'weekly' ? i18n.t('dailyReport.emptyHintWeekly') : i18n.t('dailyReport.emptyHint')}
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Project section (visible in per-project / all modes) -->
				{#if reportsStore.mode === 'per-project' || reportsStore.mode === 'all'}
					{#if projectsStore.selected}
						<div>
							{#if reportsStore.mode === 'all'}
								<div class="px-2 py-1 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
									{projectsStore.selected.name}
								</div>
							{/if}
							<div class="space-y-1 mt-1">
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
									<div class="text-xs text-muted-foreground text-center py-4 px-2">
										{reportsStore.reportPeriod === 'weekly' ? i18n.t('dailyReport.emptyHintWeekly') : i18n.t('dailyReport.emptyHint')}
									</div>
								{/each}
							</div>
						</div>
					{:else if reportsStore.mode === 'per-project'}
						<div class="flex-1 flex items-center justify-center">
							<div class="text-sm text-muted-foreground text-center px-4">
								{reportsStore.reportPeriod === 'weekly' ? i18n.t('dailyReport.emptyHintWeekly') : i18n.t('dailyReport.emptyHint')}
							</div>
						</div>
					{/if}
				{/if}
			</div>
		</ScrollArea>
	{:else}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-sm text-muted-foreground text-center px-4">
				{reportsStore.reportPeriod === 'weekly' ? i18n.t('dailyReport.emptyHintWeekly') : i18n.t('dailyReport.emptyHint')}
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

<Dialog.Root bind:open={showDateDialog}>
	<Dialog.Content class="sm:max-w-[360px]">
		<Dialog.Header>
			<Dialog.Title>{i18n.t('dailyReport.selectDate')}</Dialog.Title>
		</Dialog.Header>
		<div class="py-4 flex justify-center">
			<Calendar.Calendar
				type="single"
				bind:value={customCalendarValue}
				captionLayout="dropdown"
				class="rounded-md border"
			/>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showDateDialog = false)}>
				{i18n.t('common.cancel')}
			</Button>
			<Button onclick={confirmCustomDate}>
				{i18n.t('common.save')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
