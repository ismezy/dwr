import { invoke } from '@tauri-apps/api/core';

export interface ConfigData {
	work_dir?: string;
	git_user_name?: string;
}

async function loadConfigs(): Promise<ConfigData> {
	try {
		return await invoke<ConfigData>('get_configs');
	} catch (e) {
		console.error('failed to load configs:', e);
		return {};
	}
}

function createConfigStore() {
	let configs = $state<ConfigData>({});
	let initialized = $state(false);

	async function init() {
		if (initialized) return;
		configs = await loadConfigs();
		initialized = true;
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
		},
		async save(data: ConfigData) {
			await invoke('save_configs', { data });
			configs = data;
		},
	};
}

export const configStore = createConfigStore();
