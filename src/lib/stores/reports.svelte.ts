import { invoke } from '@tauri-apps/api/core';

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
		} catch (e) {
			console.error('failed to read report:', e);
			content = '';
		}
	}

	function selectDate(date: string | null) {
		selectedDate = date;
	}

	return {
		get reports() { return reports; },
		get selectedDate() { return selectedDate; },
		get content() { return content; },
		get loading() { return loading; },
		get generating() { return generating; },
		loadReports,
		generateReport,
		readReport,
		selectDate,
	};
}

export const reportsStore = createReportsStore();
