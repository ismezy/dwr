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
	import { CalendarDays, FileText, Sparkles, Loader, ChevronDown, ChevronRight, FolderOpen, Folder, FolderGit2 } from '@lucide/svelte';
	import { cn } from '$lib/utils';
	import * as Calendar from '$lib/components/ui/calendar/index.js';
	import { CalendarDate, type DateValue } from '@internationalized/date';
	import { untrack } from 'svelte';

	let showAiAlert = $state(false);
	let showNoProjectAlert = $state(false);
	let showGenerateMenu = $state(false);
	let showDateDialog = $state(false);
	let customCalendarValue = $state<DateValue | undefined>(undefined);
	let expandedYears = $state<Set<string>>(new Set());
	let expandedMonths = $state<Set<string>>(new Set());
	let generateMenuEl = $state<HTMLDivElement | null>(null);

	let activeDirs = $derived(projectsStore.selectedDirs);

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
		const dirs = projectsStore.selectedDirs;
		if (dirs.length === 0) return;

		if (!isAiConfigured()) {
			showAiAlert = true;
			return;
		}

		for (const dir of dirs) {
			await reportsStore.generateReport(
				dir.id,
				date,
				configStore.configs.work_dir,
				weekEnd
			);
		}
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
		for (const dir of projectsStore.selectedDirs) {
			await reportsStore.generateReport(
				dir.id,
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
		if (projectsStore.dirs.length === 0) {
			showNoProjectAlert = true;
			return;
		}
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

	async function handleSelect(dirId: string, date: string) {
		await reportsStore.readReport(
			dirId,
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

	// Tree expand/collapse
	function toggleYear(year: string) {
		const next = new Set(expandedYears);
		if (next.has(year)) {
			next.delete(year);
		} else {
			next.add(year);
		}
		expandedYears = next;
	}

	function formatYearLabel(year: string): string {
		return i18n.locale === 'en' ? year : `${year}年`;
	}

	function formatYearMonthLabel(year: string, month: string): string {
		if (i18n.locale === 'en') {
			const d = new Date(parseInt(year), parseInt(month) - 1);
			return new Intl.DateTimeFormat('en-US', { year: 'numeric', month: 'long' }).format(d);
		}
		return `${year}年${month}月`;
	}

	function formatWeeklyDateLabel(date: string): string {
		return date.replace(/至/g, ' ~ ');
	}

	function toggleMonth(yearMonth: string) {
		const next = new Set(expandedMonths);
		if (next.has(yearMonth)) {
			next.delete(yearMonth);
		} else {
			next.add(yearMonth);
		}
		expandedMonths = next;
	}

	function isYearExpanded(year: string): boolean {
		return expandedYears.has(year);
	}

	function isMonthExpanded(yearMonth: string): boolean {
		return expandedMonths.has(yearMonth);
	}

	// Group daily reports by year/month
	function groupDailyReports(reports: { date: string; path: string }[]) {
		const yearMap = new Map<string, Map<string, { date: string; path: string }[]>>();
		for (const report of reports) {
			const [y, m] = report.date.split('-');
			if (!yearMap.has(y)) yearMap.set(y, new Map());
			const monthMap = yearMap.get(y)!;
			if (!monthMap.has(m)) monthMap.set(m, []);
			monthMap.get(m)!.push(report);
		}
		// Sort descending
		return Array.from(yearMap.entries())
			.sort((a, b) => b[0].localeCompare(a[0]))
			.map(([year, monthMap]) => ({
				year,
				months: Array.from(monthMap.entries())
					.sort((a, b) => b[0].localeCompare(a[0]))
					.map(([month, items]) => ({
						month,
						items: items.sort((a, b) => b.date.localeCompare(a.date)),
					})),
			}));
	}

	// Group weekly reports by year
	function groupWeeklyReports(reports: { date: string; path: string }[]) {
		const yearMap = new Map<string, { date: string; path: string }[]>();
		for (const report of reports) {
			const year = report.date.split('-')[0];
			if (!yearMap.has(year)) yearMap.set(year, []);
			yearMap.get(year)!.push(report);
		}
		return Array.from(yearMap.entries())
			.sort((a, b) => b[0].localeCompare(a[0]))
			.map(([year, items]) => ({
				year,
				items: items.sort((a, b) => b.date.localeCompare(a.date)),
			}));
	}

	// Auto expand current year/month when reports change
	$effect(() => {
		const _period = reportsStore.reportPeriod;
		const _reports = Object.values(reportsStore.reportsByDir).flat();
		const _summary = reportsStore.summaryReports;

		const today = getTodayStr();
		const [currentYear, currentMonth] = today.split('-');

		// Use untrack to avoid self-triggering when expanded state changes
		const currentYears = untrack(() => expandedYears);
		const currentMonths = untrack(() => expandedMonths);

		let changed = false;
		const nextYears = new Set(currentYears);
		const nextMonths = new Set(currentMonths);

		if (_period === 'weekly') {
			const hasCurrentYear = _reports.some(r => r.date.startsWith(currentYear)) ||
				_summary.some(r => r.date.startsWith(currentYear));
			if (hasCurrentYear && !nextYears.has(currentYear)) {
				nextYears.add(currentYear);
				changed = true;
			}
		} else {
			const hasCurrentYear = _reports.some(r => r.date.startsWith(currentYear)) ||
				_summary.some(r => r.date.startsWith(currentYear));
			if (hasCurrentYear) {
				if (!nextYears.has(currentYear)) {
					nextYears.add(currentYear);
					changed = true;
				}
				const monthKey = `${currentYear}-${currentMonth}`;
				if (!nextMonths.has(monthKey)) {
					nextMonths.add(monthKey);
					changed = true;
				}
			}
		}

		if (changed) {
			expandedYears = nextYears;
			expandedMonths = nextMonths;
		}
	});

	$effect(() => {
		if (!showGenerateMenu) return;
		function handleDocClick(e: MouseEvent) {
			if (generateMenuEl?.contains(e.target as Node)) return;
			showGenerateMenu = false;
		}
		setTimeout(() => {
			document.addEventListener('click', handleDocClick);
		}, 10);
		return () => {
			document.removeEventListener('click', handleDocClick);
		};
	});

	$effect(() => {
		const dirs = projectsStore.selectedDirs;
		reportsStore.loadReportsForDirs(dirs, configStore.configs.work_dir);
		reportsStore.selectDate(null);
	});

	$effect(() => {
		if (reportsStore.mode === 'summary' || reportsStore.mode === 'all') {
			reportsStore.loadSummaryReports(configStore.configs.work_dir);
		}
	});

	$effect(() => {
		// Reload when report period changes
		const period = reportsStore.reportPeriod;
		const dirs = projectsStore.selectedDirs;
		if (dirs.length > 0) {
			reportsStore.loadReportsForDirs(dirs, configStore.configs.work_dir);
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
		<div class="flex-1 min-h-0 overflow-hidden">
			<ScrollArea class="h-full {isGenerating ? 'opacity-50 pointer-events-none' : ''}">
			<div class="p-2 space-y-4">
				<!-- Summary section (visible in summary / all modes) -->
				{#if reportsStore.mode === 'summary' || reportsStore.mode === 'all'}
					<div>
						<div class="px-2 py-1 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
							{reportsStore.reportPeriod === 'weekly' ? i18n.t('dailyReport.summarySectionWeekly') : i18n.t('dailyReport.summarySection')}
						</div>
						<div class="space-y-0.5 mt-1">
							{#if reportsStore.reportPeriod === 'weekly'}
								{#each groupWeeklyReports(reportsStore.summaryReports) as yearGroup}
									<div>
										<button
											class="w-full flex items-center gap-1 px-2 py-1 text-sm font-medium hover:bg-accent/50 rounded-md transition-colors"
											onclick={() => toggleYear(yearGroup.year)}
										>
											{#if isYearExpanded(yearGroup.year)}
												<FolderOpen class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
												<ChevronDown class="h-3 w-3 shrink-0 text-muted-foreground" />
											{:else}
												<Folder class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
												<ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
											{/if}
											<span>{formatYearLabel(yearGroup.year)}</span>
										</button>
										{#if isYearExpanded(yearGroup.year)}
											<div class="pl-5 space-y-0.5">
												{#each yearGroup.items as report}
													<button
														class={cn(
															'w-full text-left rounded-md px-3 py-1.5 transition-colors flex items-center gap-2 text-sm',
															reportsStore.summarySelectedDate === report.date
																? 'bg-accent text-accent-foreground'
																: 'hover:bg-accent/50 text-foreground'
														)}
														onclick={() => handleSelectSummary(report.date)}
													>
														<FileText class="h-3.5 w-3.5 shrink-0 opacity-60" />
														<span>{report.date}</span>
													</button>
												{/each}
											</div>
										{/if}
									</div>
								{:else}
									<div class="text-xs text-muted-foreground text-center py-4 px-2">
										{i18n.t('dailyReport.emptyHintWeekly')}
									</div>
								{/each}
							{:else}
								{#each groupDailyReports(reportsStore.summaryReports) as yearGroup}
									<div>
										<button
											class="w-full flex items-center gap-1 px-2 py-1 text-sm font-medium hover:bg-accent/50 rounded-md transition-colors"
											onclick={() => toggleYear(yearGroup.year)}
										>
											{#if isYearExpanded(yearGroup.year)}
												<FolderOpen class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
												<ChevronDown class="h-3 w-3 shrink-0 text-muted-foreground" />
											{:else}
												<Folder class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
												<ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
											{/if}
											<span>{formatYearLabel(yearGroup.year)}</span>
										</button>
										{#if isYearExpanded(yearGroup.year)}
											<div class="pl-5 space-y-0.5">
												{#each yearGroup.months as monthGroup}
													<div>
														<button
															class="w-full flex items-center gap-1 px-2 py-1 text-sm hover:bg-accent/50 rounded-md transition-colors"
															onclick={() => toggleMonth(`${yearGroup.year}-${monthGroup.month}`)}
														>
															{#if isMonthExpanded(`${yearGroup.year}-${monthGroup.month}`)}
																<FolderOpen class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
																<ChevronDown class="h-3 w-3 shrink-0 text-muted-foreground" />
															{:else}
																<Folder class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
																<ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
															{/if}
															<span>{formatYearMonthLabel(yearGroup.year, monthGroup.month)}</span>
														</button>
														{#if isMonthExpanded(`${yearGroup.year}-${monthGroup.month}`)}
															<div class="pl-5 space-y-0.5">
																{#each monthGroup.items as report}
																	<button
																		class={cn(
																			'w-full text-left rounded-md px-3 py-1.5 transition-colors flex items-center gap-2 text-sm',
																			reportsStore.summarySelectedDate === report.date
																				? 'bg-accent text-accent-foreground'
																				: 'hover:bg-accent/50 text-foreground'
																		)}
																		onclick={() => handleSelectSummary(report.date)}
																	>
																		<FileText class="h-3.5 w-3.5 shrink-0 opacity-60" />
																		<span>{report.date}</span>
																	</button>
																{/each}
															</div>
														{/if}
													</div>
												{/each}
											</div>
										{/if}
									</div>
								{:else}
									<div class="text-xs text-muted-foreground text-center py-4 px-2">
										{i18n.t('dailyReport.emptyHint')}
									</div>
								{/each}
							{/if}
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
							{#if activeDirs.length === 0}
								<div class="text-xs text-muted-foreground text-center py-4 px-2">
									{i18n.t('project.emptyDirHint')}
								</div>
							{/if}
							{#each activeDirs as dir (dir.id)}
								{#if activeDirs.length > 1}
									<div class="px-2 py-1 text-xs font-medium text-muted-foreground flex items-center gap-1.5 mt-1">
										{#if dir.project_type === 'docs'}
											<FileText class="h-3.5 w-3.5 shrink-0" />
										{:else}
											<FolderGit2 class="h-3.5 w-3.5 shrink-0" />
										{/if}
										<span class="truncate">{dir.name}</span>
									</div>
								{/if}
							<div class="space-y-0.5 mt-1">
								{#if reportsStore.reportPeriod === 'weekly'}
									{#each groupWeeklyReports(reportsStore.reportsByDir[dir.id] ?? []) as yearGroup}
										<div>
											<button
												class="w-full flex items-center gap-1 px-2 py-1 text-sm font-medium hover:bg-accent/50 rounded-md transition-colors"
												onclick={() => toggleYear(yearGroup.year)}
											>
												{#if isYearExpanded(yearGroup.year)}
													<FolderOpen class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
													<ChevronDown class="h-3 w-3 shrink-0 text-muted-foreground" />
												{:else}
													<Folder class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
													<ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
												{/if}
												<span>{formatYearLabel(yearGroup.year)}</span>
											</button>
											{#if isYearExpanded(yearGroup.year)}
												<div class="pl-5 space-y-0.5">
													{#each yearGroup.items as report}
														<button
															class={cn(
																'w-full text-left rounded-md px-3 py-1.5 transition-colors flex items-center gap-2 text-sm',
																reportsStore.selectedDirId === dir.id && reportsStore.selectedDate === report.date
																	? 'bg-accent text-accent-foreground'
																	: 'hover:bg-accent/50 text-foreground'
															)}
															onclick={() => handleSelect(dir.id, report.date)}
														>
															<FileText class="h-3.5 w-3.5 shrink-0 opacity-60" />
															<span>{report.date}</span>
														</button>
													{/each}
												</div>
											{/if}
										</div>
									{:else}
										<div class="text-xs text-muted-foreground text-center py-4 px-2">
											{i18n.t('dailyReport.emptyHintWeekly')}
										</div>
									{/each}
								{:else}
									{#each groupDailyReports(reportsStore.reportsByDir[dir.id] ?? []) as yearGroup}
										<div>
											<button
												class="w-full flex items-center gap-1 px-2 py-1 text-sm font-medium hover:bg-accent/50 rounded-md transition-colors"
												onclick={() => toggleYear(yearGroup.year)}
											>
												{#if isYearExpanded(yearGroup.year)}
													<FolderOpen class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
													<ChevronDown class="h-3 w-3 shrink-0 text-muted-foreground" />
												{:else}
													<Folder class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
													<ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
												{/if}
												<span>{formatYearLabel(yearGroup.year)}</span>
											</button>
											{#if isYearExpanded(yearGroup.year)}
												<div class="pl-5 space-y-0.5">
													{#each yearGroup.months as monthGroup}
														<div>
															<button
																class="w-full flex items-center gap-1 px-2 py-1 text-sm hover:bg-accent/50 rounded-md transition-colors"
																onclick={() => toggleMonth(`${yearGroup.year}-${monthGroup.month}`)}
															>
																{#if isMonthExpanded(`${yearGroup.year}-${monthGroup.month}`)}
																	<FolderOpen class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
																	<ChevronDown class="h-3 w-3 shrink-0 text-muted-foreground" />
																{:else}
																	<Folder class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
																	<ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
																{/if}
																<span>{formatYearMonthLabel(yearGroup.year, monthGroup.month)}</span>
															</button>
															{#if isMonthExpanded(`${yearGroup.year}-${monthGroup.month}`)}
																<div class="pl-5 space-y-0.5">
																	{#each monthGroup.items as report}
																		<button
																			class={cn(
																				'w-full text-left rounded-md px-3 py-1.5 transition-colors flex items-center gap-2 text-sm',
																				reportsStore.selectedDirId === dir.id && reportsStore.selectedDate === report.date
																					? 'bg-accent text-accent-foreground'
																					: 'hover:bg-accent/50 text-foreground'
																				)}
																			onclick={() => handleSelect(dir.id, report.date)}
																		>
																			<FileText class="h-3.5 w-3.5 shrink-0 opacity-60" />
																			<span>{report.date}</span>
																		</button>
																	{/each}
																</div>
															{/if}
														</div>
													{/each}
												</div>
											{/if}
										</div>
									{:else}
										<div class="text-xs text-muted-foreground text-center py-4 px-2">
											{i18n.t('dailyReport.emptyHint')}
										</div>
									{/each}
								{/if}
							</div>
							{/each}
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
		</div>
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

<AlertDialog.Root bind:open={showNoProjectAlert}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{i18n.t('dailyReport.noProjectTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				{i18n.t('dailyReport.noProjectMessage')}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel onclick={() => (showNoProjectAlert = false)}>
				{i18n.t('common.cancel')}
			</AlertDialog.Cancel>
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
