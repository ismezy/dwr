import { invoke } from '@tauri-apps/api/core';
import { untrack } from 'svelte';
import { i18n } from '$lib/i18n';
import { projectsStore, type Project } from './projects.svelte';

export type ReportMode = 'per-project' | 'summary' | 'all';
export type ReportPeriod = 'daily' | 'weekly';

export interface ReportMeta {
	date: string;
	path: string;
}

export interface DailyReport {
	date: string;
	content: string;
}

export function getWeekRange(dateStr: string, weekStartDay: number = 1): { start: string; end: string } {
	const date = new Date(dateStr + 'T00:00:00');
	const jsDay = date.getDay(); // 0=Sunday, 1=Monday, ...
	const currentDay = jsDay === 0 ? 7 : jsDay; // 1=Monday, ..., 7=Sunday
	const diff = currentDay - weekStartDay;
	const start = new Date(date);
	start.setDate(date.getDate() - diff);
	const end = new Date(start);
	end.setDate(start.getDate() + 6);
	return {
		start: `${start.getFullYear()}-${String(start.getMonth() + 1).padStart(2, '0')}-${String(start.getDate()).padStart(2, '0')}`,
		end: `${end.getFullYear()}-${String(end.getMonth() + 1).padStart(2, '0')}-${String(end.getDate()).padStart(2, '0')}`,
	};
}

function getInitialMode(): ReportMode {
	try {
		const saved = localStorage.getItem('dwr:report-mode');
		if (saved === 'per-project' || saved === 'summary' || saved === 'all') return saved;
	} catch {
		// noop: localStorage may not be available in release builds
	}
	return 'summary';
}

function createReportsStore() {
	let reportsByDir = $state<Record<string, ReportMeta[]>>({});
	let selectedDirId = $state<string | null>(null);
	let selectedDate = $state<string | null>(null);
	let content = $state<string>('');
	let loading = $state(false);
	let generating = $state(false);

	let mode = $state<ReportMode>(getInitialMode());
	let reportPeriod = $state<ReportPeriod>('daily');

	let summaryReports = $state<ReportMeta[]>([]);
	let summarySelectedDate = $state<string | null>(null);
	let summaryContent = $state<string>('');
	let summaryLoading = $state(false);
	let summaryGenerating = $state(false);

	async function loadReports(dirId: string, workDir?: string) {
		const dir = projectsStore.byId(dirId);
		if (!dir || dir.parent_id == null) return;
		loading = true;
		try {
			let list: ReportMeta[];
			if (reportPeriod === 'weekly') {
				list = await invoke<ReportMeta[]>('get_weekly_report_list', {
					projectPath: dir.path,
					projectName: dir.name,
					workDir,
				});
			} else {
				list = await invoke<ReportMeta[]>('get_report_list', {
					projectPath: dir.path,
					projectName: dir.name,
					workDir,
				});
			}
			reportsByDir = { ...reportsByDir, [dirId]: list };
		} catch (e) {
			console.error('failed to load reports:', e);
			reportsByDir = { ...reportsByDir, [dirId]: [] };
		} finally {
			loading = false;
		}
	}

	async function loadReportsForDirs(dirs: Project[], workDir?: string) {
		// 清掉不再选中的目录缓存，避免陈旧数据
		// 注意：这里读写 reportsByDir 必须 untrack，否则调用方（$effect）会订阅它，
		// loadReports 完成后写回又会重新触发 effect，造成无限循环
		const keep = new Set(dirs.map((d) => d.id));
		untrack(() => {
			const next: Record<string, ReportMeta[]> = {};
			for (const [k, v] of Object.entries(reportsByDir)) {
				if (keep.has(k)) next[k] = v;
			}
			reportsByDir = next;
		});
		await Promise.all(dirs.map((d) => loadReports(d.id, workDir)));
	}

	async function generateReport(dirId: string, date: string, workDir?: string, weekEnd?: string) {
		const dir = projectsStore.byId(dirId);
		if (!dir || dir.parent_id == null) return;
		const gitUserName = projectsStore.resolveGitUserName(dir);
		generating = true;
		try {
			if (reportPeriod === 'weekly') {
				const report = await invoke<DailyReport>('generate_weekly_report', {
					projectPath: dir.path,
					projectName: dir.name,
					gitUserName,
					weekStart: date,
					weekEnd,
					workDir,
					locale: i18n.locale,
				});
				await loadReports(dirId, workDir);
				selectedDirId = dirId;
				selectedDate = date;
				summarySelectedDate = null;
				content = report.content;
			} else {
				const report = await invoke<DailyReport>('generate_daily_report', {
					projectPath: dir.path,
					projectName: dir.name,
					gitUserName,
					date,
					workDir,
					locale: i18n.locale,
					projectType: dir.project_type,
				});
				await loadReports(dirId, workDir);
				selectedDirId = dirId;
				selectedDate = date;
				summarySelectedDate = null;
				content = report.content;
			}
		} catch (e) {
			console.error('failed to generate report:', e);
			throw e;
		} finally {
			generating = false;
		}
	}

	async function readReport(dirId: string, date: string, workDir?: string) {
		const dir = projectsStore.byId(dirId);
		if (!dir || dir.parent_id == null) return;
		selectedDirId = dirId;
		selectedDate = date;
		summarySelectedDate = null;
		try {
			if (reportPeriod === 'weekly') {
				content = await invoke<string>('read_weekly_report', {
					projectPath: dir.path,
					projectName: dir.name,
					weekStart: date,
					workDir,
				});
			} else {
				content = await invoke<string>('read_report', {
					projectPath: dir.path,
					projectName: dir.name,
					date,
					workDir,
					locale: i18n.locale,
				});
			}
		} catch (e) {
			console.error('failed to read report:', e);
			content = '';
		}
	}

	function selectDate(date: string | null) {
		selectedDate = date;
		summarySelectedDate = null;
	}

	async function loadSummaryReports(workDir?: string) {
		summaryLoading = true;
		try {
			if (reportPeriod === 'weekly') {
				summaryReports = await invoke<ReportMeta[]>('get_weekly_summary_report_list', {
					workDir,
				});
			} else {
				summaryReports = await invoke<ReportMeta[]>('get_summary_report_list', {
					workDir,
				});
			}
		} catch (e) {
			console.error('failed to load summary reports:', e);
			summaryReports = [];
		} finally {
			summaryLoading = false;
		}
	}

	async function generateSummaryReport(date: string, workDir?: string, weekEnd?: string) {
		summaryGenerating = true;
		try {
			if (reportPeriod === 'weekly') {
				const report = await invoke<DailyReport>('generate_weekly_summary_report', {
					weekStart: date,
					weekEnd,
					workDir,
					locale: i18n.locale,
				});
				await loadSummaryReports(workDir);
				summarySelectedDate = date;
				selectedDate = null;
				summaryContent = report.content;
			} else {
				const report = await invoke<DailyReport>('generate_summary_report', {
					date,
					workDir,
				});
				await loadSummaryReports(workDir);
				summarySelectedDate = date;
				selectedDate = null;
				summaryContent = report.content;
			}
		} catch (e) {
			console.error('failed to generate summary report:', e);
			throw e;
		} finally {
			summaryGenerating = false;
		}
	}

	async function readSummaryReport(date: string, workDir?: string) {
		summarySelectedDate = date;
		selectedDate = null;
		try {
			if (reportPeriod === 'weekly') {
				summaryContent = await invoke<string>('read_weekly_summary_report', {
					weekStart: date,
					workDir,
				});
			} else {
				summaryContent = await invoke<string>('read_summary_report', {
					date,
					workDir,
				});
			}
		} catch (e) {
			console.error('failed to read summary report:', e);
			summaryContent = '';
		}
	}

	async function saveReport(
		projectPath: string,
		projectName: string,
		date: string,
		content: string,
		workDir?: string
	) {
		if (reportPeriod === 'weekly') {
			await invoke('save_weekly_report', {
				projectPath,
				projectName,
				weekStart: date,
				content,
				workDir,
			});
		} else {
			await invoke('save_report', {
				projectPath,
				projectName,
				date,
				content,
				workDir,
			});
		}
	}

	async function saveSummaryReport(date: string, content: string, workDir?: string) {
		if (reportPeriod === 'weekly') {
			await invoke('save_weekly_summary_report', {
				weekStart: date,
				content,
				workDir,
			});
		} else {
			await invoke('save_summary_report', {
				date,
				content,
				workDir,
			});
		}
	}

	async function polish(content: string) {
		return await invoke<string>('polish_report', { content, locale: i18n.locale });
	}

	async function refine(content: string, instruction: string) {
		return await invoke<string>('refine_report', {
			content,
			instruction,
			locale: i18n.locale,
		});
	}

	function selectSummaryDate(date: string | null) {
		summarySelectedDate = date;
		selectedDate = null;
	}

	function setMode(newMode: ReportMode) {
		mode = newMode;
		try {
			localStorage.setItem('dwr:report-mode', newMode);
		} catch {
			// noop: localStorage may not be available in release builds
		}
	}

	function setReportPeriod(period: ReportPeriod) {
		reportPeriod = period;
		// Reset selections when switching period
		reportsByDir = {};
		selectedDirId = null;
		selectedDate = null;
		summarySelectedDate = null;
		content = '';
		summaryContent = '';
	}

	return {
		get reportsByDir() { return reportsByDir; },
		get selectedDirId() { return selectedDirId; },
		get selectedDate() { return selectedDate; },
		get content() { return content; },
		set content(value: string) { content = value; },
		get loading() { return loading; },
		get generating() { return generating; },
		get mode() { return mode; },
		get reportPeriod() { return reportPeriod; },
		get summaryReports() { return summaryReports; },
		get summarySelectedDate() { return summarySelectedDate; },
		get summaryContent() { return summaryContent; },
		set summaryContent(value: string) { summaryContent = value; },
		get summaryLoading() { return summaryLoading; },
		get summaryGenerating() { return summaryGenerating; },
		loadReports,
		loadReportsForDirs,
		generateReport,
		readReport,
		selectDate,
		loadSummaryReports,
		generateSummaryReport,
		readSummaryReport,
		selectSummaryDate,
		setMode,
		setReportPeriod,
		saveReport,
		saveSummaryReport,
		polish,
		refine,
	};
}

export const reportsStore = createReportsStore();
