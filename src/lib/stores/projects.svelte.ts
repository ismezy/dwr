import { invoke } from '@tauri-apps/api/core';
import { configStore } from './config.svelte';

export interface Project {
	id: string;
	name: string;
	code?: string;
	path: string;
	git_user_name?: string;
}

async function loadProjects(): Promise<Project[]> {
	try {
		return await invoke<Project[]>('get_projects');
	} catch (e) {
		console.error('failed to load projects:', e);
		return [];
	}
}

function createProjectsStore() {
	let projects = $state<Project[]>([]);
	let selectedId = $state<string | null>(null);
	let initialized = $state(false);

	async function init() {
		if (initialized) return;
		projects = await loadProjects();
		initialized = true;
		if (projects.length > 0 && !selectedId) {
			selectedId = projects[0].id;
		}
	}

	function resolveGitUserName(project: Project | null): string | undefined {
		if (!project) return undefined;
		return project.git_user_name ?? configStore.configs.git_user_name;
	}

	return {
		get projects() {
			if (!initialized) init();
			return projects;
		},
		get selectedId() {
			return selectedId;
		},
		get selected() {
			return projects.find((p) => p.id === selectedId) ?? null;
		},
		get initialized() {
			return initialized;
		},
		select(id: string | null) {
			selectedId = id;
		},
		async add(project: Omit<Project, 'id'>) {
			const created = await invoke<Project>('create_project', {
				name: project.name,
				code: project.code,
				path: project.path,
				git_user_name: project.git_user_name,
			});
			projects = [...projects, created];
			selectedId = created.id;
			return created;
		},
		async update(id: string, patch: Partial<Omit<Project, 'id'>>) {
			const current = projects.find((p) => p.id === id);
			if (!current) return;
			const updated = await invoke<Project>('update_project', {
				id,
				name: patch.name ?? current.name,
				code: patch.code ?? current.code,
				path: patch.path ?? current.path,
				git_user_name: patch.git_user_name ?? current.git_user_name,
			});
			projects = projects.map((p) => (p.id === id ? updated : p));
			return updated;
		},
		async remove(id: string) {
			await invoke<void>('delete_project', { id });
			const next = projects.filter((p) => p.id !== id);
			projects = next;
			if (selectedId === id) {
				selectedId = next[0]?.id ?? null;
			}
		},
		resolveGitUserName,
	};
}

export const projectsStore = createProjectsStore();
