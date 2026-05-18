import { invoke } from '@tauri-apps/api/core';
import { i18n, type Locale } from '$lib/i18n';

export type Theme = 'light' | 'dark' | 'system';
export type AiProvider = 'openai' | 'anthropic' | 'gemini' | 'custom' | '';

export interface ConfigData {
	work_dir?: string;
	git_user_name?: string;
	lang?: Locale;
	theme?: Theme;
	ai_provider?: string;
	ai_api_key?: string;
	ai_base_url?: string;
	ai_model?: string;
}

async function loadConfigs(): Promise<ConfigData> {
	try {
		return await invoke<ConfigData>('get_configs');
	} catch (e) {
		console.error('failed to load configs:', e);
		return {};
	}
}

function applyTheme(theme?: Theme) {
	const t = theme ?? 'system';
	const root = document.documentElement;
	const media = window.matchMedia('(prefers-color-scheme: dark)');
	if (t === 'dark') {
		root.classList.add('dark');
	} else if (t === 'light') {
		root.classList.remove('dark');
	} else {
		if (media.matches) {
			root.classList.add('dark');
		} else {
			root.classList.remove('dark');
		}
	}
}

function createConfigStore() {
	let configs = $state<ConfigData>({});
	let initialized = $state(false);

	async function init() {
		if (initialized) return;
		configs = await loadConfigs();
		initialized = true;
		if (configs.lang) {
			i18n.setLocale(configs.lang);
		}
		applyTheme(configs.theme);
	}

	return {
		get configs() {
			if (!initialized) init();
			return configs;
		},
		get initialized() {
			return initialized;
		},
		async refresh() {
			configs = await loadConfigs();
			initialized = true;
			if (configs.lang) {
				i18n.setLocale(configs.lang);
			}
			applyTheme(configs.theme);
		},
		async save(data: ConfigData) {
			await invoke('save_configs', { data });
			configs = data;
			if (data.lang) {
				i18n.setLocale(data.lang);
			}
			applyTheme(data.theme);
		},
		applyTheme,
	};
}

export const configStore = createConfigStore();
