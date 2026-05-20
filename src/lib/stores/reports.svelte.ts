import { invoke } from '@tauri-apps/api/core';
import { i18n } from '$lib/i18n';

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

function createReportsStore() {
	let reports = $state<ReportMeta[]>([]);
	let selectedDate = $state<string | null>(null);
	let content = $state<string>('');
	let loading = $state(false);
	let generating = $state(false);

	let mode = $state<ReportMode>('per-project');
	let reportPeriod = $state<ReportPeriod>('daily');

	let summaryReports = $state<ReportMeta[]>([]);
	let summarySelectedDate = $state<string | null>(null);
	let summaryContent = $state<string>('');
	let summaryLoading = $state(false);
	let summaryGenerating = $state(false);

	async function loadReports(projectPath: string, projectName: string, workDir?: string) {
		loading = true;
		try {
			if (reportPeriod === 'weekly') {
				reports = await invoke<ReportMeta[]>('get_weekly_report_list', {
					projectPath,
					projectName,
					workDir,
				});
			} else {
				reports = await invoke<ReportMeta[]>('get_report_list', {
					projectPath,
					projectName,
					workDir,
				});
			}
		} catch (e) {
			console.error('failed to load reports:', e);
			reports = [];
		} finally {
			loading = false;
		}
	}

	async function generateReport(
		projectPath: string,
		projectName: string,
		gitUserName: string | undefined,
		date: string,
		workDir?: string,
		weekEnd?: string
	) {
		generating = true;
		try {
			if (reportPeriod === 'weekly') {
				const report = await invoke<DailyReport>('generate_weekly_report', {
					projectPath,
					projectName,
					gitUserName,
					weekStart: date,
					weekEnd,
					workDir,
					locale: i18n.locale,
				});
				await loadReports(projectPath, projectName, workDir);
				selectedDate = date;
				summarySelectedDate = null;
				content = report.content;
			} else {
				const report = await invoke<DailyReport>('generate_daily_report', {
					projectPath,
					projectName,
					gitUserName,
					date,
					workDir,
					locale: i18n.locale,
				});
				await loadReports(projectPath, projectName, workDir);
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

	async function readReport(
		projectPath: string,
		projectName: string,
		date: string,
		workDir?: string
	) {
		selectedDate = date;
		summarySelectedDate = null;
		try {
			if (reportPeriod === 'weekly') {
				content = await invoke<string>('read_weekly_report', {
					projectPath,
					projectName,
					weekStart: date,
					workDir,
				});
			} else {
				content = await invoke<string>('read_report', {
					projectPath,
					projectName,
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

	function selectSummaryDate(date: string | null) {
		summarySelectedDate = date;
		selectedDate = null;
	}

	function setMode(newMode: ReportMode) {
		mode = newMode;
	}

	function setReportPeriod(period: ReportPeriod) {
		reportPeriod = period;
		// Reset selections when switching period
		selectedDate = null;
		summarySelectedDate = null;
		content = '';
		summaryContent = '';
	}

	return {
		get reports() { return reports; },
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
	};
}

export const reportsStore = createReportsStore();
