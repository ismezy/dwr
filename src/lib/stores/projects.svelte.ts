import { invoke } from '@tauri-apps/api/core';
import { configStore } from './config.svelte';

export interface Project {
	id: string;
	name: string;
	code?: string;
	path: string;
	git_user_name?: string;
	project_type: 'code' | 'docs';
	parent_id?: string | null;
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
			const groups = projects.filter((p) => p.parent_id == null);
			selectedId = (groups[0] ?? projects[0]).id;
		}
	}

	function resolveGitUserName(project: Project | null): string | undefined {
		if (!project) return undefined;
		return project.git_user_name ?? configStore.configs.git_user_name;
	}

	function dirsOf(groupId: string): Project[] {
		return projects.filter((p) => p.parent_id === groupId);
	}

	return {
		get projects() {
			if (!initialized) init();
			return projects;
		},
		get groups() {
			if (!initialized) init();
			return projects.filter((p) => p.parent_id == null);
		},
		get dirs() {
			return projects.filter((p) => p.parent_id != null);
		},
		get selectedId() {
			return selectedId;
		},
		get selected() {
			return projects.find((p) => p.id === selectedId) ?? null;
		},
		get selectedDirs(): Project[] {
			const sel = projects.find((p) => p.id === selectedId);
			if (!sel) return [];
			if (sel.parent_id == null) return dirsOf(sel.id);
			return [sel];
		},
		get initialized() {
			return initialized;
		},
		dirsOf,
		byId(id: string | null | undefined): Project | null {
			return projects.find((p) => p.id === id) ?? null;
		},
		select(id: string | null) {
			selectedId = id;
		},
		async add(project: Omit<Project, 'id'>) {
			const created = await invoke<Project>('create_project', {
				name: project.name,
				code: project.code,
				path: project.path,
				gitUserName: project.git_user_name,
				projectType: project.project_type,
				parentId: project.parent_id,
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
				gitUserName: patch.git_user_name ?? current.git_user_name,
				projectType: patch.project_type ?? current.project_type,
				parentId: patch.parent_id !== undefined ? patch.parent_id : current.parent_id,
			});
			projects = projects.map((p) => (p.id === id ? updated : p));
			return updated;
		},
		async remove(id: string) {
			await invoke<void>('delete_project', { id });
			const removed = projects.find((p) => p.id === id);
			const next = projects.filter((p) => p.id !== id);
			projects = next;
			if (selectedId === id) {
				if (removed?.parent_id) {
					selectedId = removed.parent_id;
				} else {
					const groups = next.filter((p) => p.parent_id == null);
					selectedId = (groups[0] ?? next[0])?.id ?? null;
				}
			}
		},
		resolveGitUserName,
	};
}

export const projectsStore = createProjectsStore();
