import { invoke } from '@tauri-apps/api/core';

export type ReportMode = 'per-project' | 'summary' | 'all';

export interface ReportMeta {
	date: string;
	path: string;
}

export interface DailyReport {
	date: string;
	content: string;
}

function createReportsStore() {
	let reports = $state<ReportMeta[]>([]);
	let selectedDate = $state<string | null>(null);
	let content = $state<string>('');
	let loading = $state(false);
	let generating = $state(false);

	let mode = $state<ReportMode>('per-project');

	let summaryReports = $state<ReportMeta[]>([]);
	let summarySelectedDate = $state<string | null>(null);
	let summaryContent = $state<string>('');
	let summaryLoading = $state(false);
	let summaryGenerating = $state(false);

	async function loadReports(projectPath: string, projectName: string, workDir?: string) {
		loading = true;
		try {
			reports = await invoke<ReportMeta[]>('get_report_list', {
				projectPath,
				projectName,
				workDir,
			});
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
		workDir?: string
	) {
		generating = true;
		try {
			const report = await invoke<DailyReport>('generate_daily_report', {
				projectPath,
				projectName,
				gitUserName,
				date,
				workDir,
			});
			await loadReports(projectPath, projectName, workDir);
			selectedDate = date;
			summarySelectedDate = null;
			content = report.content;
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
		try {
			content = await invoke<string>('read_report', {
				projectPath,
				projectName,
				date,
				workDir,
			});
			selectedDate = date;
			summarySelectedDate = null;
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
			summaryReports = await invoke<ReportMeta[]>('get_summary_report_list', {
				workDir,
			});
		} catch (e) {
			console.error('failed to load summary reports:', e);
			summaryReports = [];
		} finally {
			summaryLoading = false;
		}
	}

	async function generateSummaryReport(date: string, workDir?: string) {
		summaryGenerating = true;
		try {
			const report = await invoke<DailyReport>('generate_summary_report', {
				date,
				workDir,
			});
			await loadSummaryReports(workDir);
			summarySelectedDate = date;
			selectedDate = null;
			summaryContent = report.content;
		} catch (e) {
			console.error('failed to generate summary report:', e);
			throw e;
		} finally {
			summaryGenerating = false;
		}
	}

	async function readSummaryReport(date: string, workDir?: string) {
		try {
			summaryContent = await invoke<string>('read_summary_report', {
				date,
				workDir,
			});
			summarySelectedDate = date;
			selectedDate = null;
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
		await invoke('save_report', {
			projectPath,
			projectName,
			date,
			content,
			workDir,
		});
	}

	async function saveSummaryReport(date: string, content: string, workDir?: string) {
		await invoke('save_summary_report', {
			date,
			content,
			workDir,
		});
	}

	async function polish(content: string) {
		return await invoke<string>('polish_report', { content });
	}

	function selectSummaryDate(date: string | null) {
		summarySelectedDate = date;
		selectedDate = null;
	}

	function setMode(newMode: ReportMode) {
		mode = newMode;
	}

	return {
		get reports() { return reports; },
		get selectedDate() { return selectedDate; },
		get content() { return content; },
		get loading() { return loading; },
		get generating() { return generating; },
		get mode() { return mode; },
		get summaryReports() { return summaryReports; },
		get summarySelectedDate() { return summarySelectedDate; },
		get summaryContent() { return summaryContent; },
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
		saveReport,
		saveSummaryReport,
		polish,
	};
}

export const reportsStore = createReportsStore();
